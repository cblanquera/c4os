# Task 00021 — r013 Artifact And Responsive Convergence

Status: verified

Coverage: corrective ownership for CHAT-008 and ART-001 through ART-007; support for UI-004, UI-005, and UX-008.

## Summary

Converge File, Folder, Browser, and Terminal response artifacts plus focus/contextual layouts on r013's shared visual and interaction contract. Preserve native authority, process/browser isolation, immutable artifact history, and conflict-safe effects.

## Implementation Steps

1. Align shared response-artifact preamble, identity, frame, actions, widths, radii, status, and internal scrolling with r013 across File, Folder, Browser, and Terminal.
2. Preserve one explicit center focus surface with no right panel or tabs. Move the real transcript into the contextual left Chat pane and keep the fixed composer in place.
3. Match r013 focused and contextual geometry, direct Close/Restore controls, pane height/width resizing, collapsed-panel behavior, and artifact-to-artifact swaps.
4. Retain File edit/proposal/conflict/version states, Folder navigation, Browser controls/permission states, and Terminal running/completed/interrupted/recovered states without recreating provider-specific outer shells.
5. Apply practical direct-intent treatment to exact user editing/navigation/process controls while preserving approval for delegated or policy-selected effects and all native isolation boundaries.
6. Exercise wide, minimum-center, overlay, narrow Settings-adjacent, zoom/text scaling, long path/content, internal scroll, and reduced-motion behavior.

## Verification Process

- Provider-contract and integration tests for inline/focused/contextual identity, state continuity, direct operations, Reply snapshots, approvals, conflicts, recovery, and no authority regression.
- Playwright and native walkthroughs for every artifact provider in ordinary, loading, approval, denied, error, focused, contextual, restored, and responsive states.
- Geometry, resize, focus, keyboard, overflow, console, process, Browser isolation, and secret/path-redaction checks.

## Acceptance Criteria

The user reviews each artifact provider inline, focused, and in the contextual Chat pane, including representative edit/navigation/process, approval/denial, error/recovery, and narrow-layout states.

## Implementation Notes

Started 2026-07-27 after Tasks 00017 and 00020 reached component-verification checkpoints. The convergence pass is retaining the production provider shells and Rust-owned artifact/session state while correcting only the remaining shared focus, contextual Chat, resizing, recovery, and responsive composition gaps. Task 00016's direct-intent policy remains the authority for exact user editing, navigation, and process controls.

The completed component pass preserves the one stable transcript portal while moving it between center and contextual Chat, retains provider-owned direct Close plus Restore Chat, and leaves Detach Chat visibly unavailable. Contextual Chat now defaults to 40% of the live viewport, retains its requested height across focus and artifact swaps, exposes one top horizontal separator for pointer and keyboard resizing, clamps to the accepted 60% maximum, and reconciles its value when the window height changes. The existing left-panel reducer remains the single width owner and continues to enforce 228px default, 180px minimum, the smaller of 55% or the width preserving the 420px center, overlay mode below 992px, and collapsed-panel recovery through focused provider Close.

File, Folder, Browser, and Terminal continue through the shared static-header/body/static-footer shell. The shared shell now uses coherent provider icons instead of a placeholder glyph and reserves subdued Copy/Reply geometry until hover or keyboard focus. Contextual-provider rules collapse wide header/metadata layouts, remove the Folder listing's fixed compact-pane minimum, and wrap Browser/Terminal controls without changing provider state or action callbacks. Focused File edit/proposal/approval/conflict/recovery, Folder navigation, Browser controller generations/profile isolation, and Terminal process generations/retained output remain unchanged and authoritative.

## Verification Notes

Component and contract verification passed on 2026-07-27:

- `npx vitest run tests/frontend/features/artifacts tests/frontend/features/conversation/focus tests/frontend/features/shell/ui/ShellView.test.tsx tests/frontend/features/shell/ui/shell-view.style.test.ts`: 13 files and 69 checks passed, including shared provider contexts, one-transcript DOM continuity, direct Close/Restore, keyboard contextual-pane resize, overlay geometry, and static artifact scrolling boundaries.
- `npm run typecheck` and `npm run lint`: passed.
- `npm run build:web`: passed with the existing non-failing large-chunk advisory.
- `cargo test --test native_artifact_acceptance --test execution_terminal --test browser_profiles`: Browser profile/isolation 9/9 and Terminal execution/process 11/11 passed; the four explicit historical native seeding tests remained ignored by design.

The integrated Task 00022 pass completed 47/47 Playwright checks; current File proposal/conflict/focused, Folder inline/overlay, Browser contextual/overlay, and Terminal wide/narrow representatives; zero browser console warnings/errors; complete Rust provider/native authority tests; QA-native Chat/Settings composition; and final production bundle/QA-boundary checks. Provider-specific approval/denial/error/recovery/restart evidence remains layered through the unchanged Task 00008–00010 native acceptance records rather than being overstated as freshly recaptured. See `.agents/resources/native/task-00022-review-package.md`.

## Acceptance Notes

Integrated verification passed. Explicit user review of each provider's shared shell and the representative focused/contextual/responsive evidence remains pending.
