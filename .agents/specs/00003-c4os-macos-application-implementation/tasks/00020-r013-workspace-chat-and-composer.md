# Task 00020 — r013 Workspace, Chat, And Composer

Status: verified

Coverage: corrective ownership for UX-005 through UX-007 and CHAT-001 through CHAT-007; support for UI-004, UI-005, UX-012, and ART-006.

## Summary

Converge the production Workspace, Chat transcript, composer, search, model controls, capability feedback, and response presentation on r013 while preserving authoritative service state and direct-operation boundaries.

## Implementation Steps

1. Restore the compact project/session navigation, centered thread title, resizable panel, flat search-result replacement, and pending-Chat hierarchy without changing durable Workspace/Session ownership.
2. Place the real Composer inside the shared raised 760px shell. Restore the accepted input/toolbar hierarchy, upward mode popover, Attach ordering, model, approval, conditional Branch, optional reasoning, and primary action.
3. Implement r013 provider/model navigation in one popover: current-provider header, in-place provider list, provider-scoped return, capability filters, active-model preservation, Escape, and focus restoration.
4. Keep compatibility feedback above the composer input and controls. Block only unresolved incompatible content; never disable ordinary text chat because of unrelated unknown capability.
5. Recompute model-dependent controls atomically, hide unsupported reasoning, preserve draft/attachments during conflicts, and offer Use compatible model, Convert, Remove file, and Cancel.
6. Match r013 transcript hierarchy for user/assistant alignment, Activity versus supplied Reasoning, model identity, contextual actions, Markdown, streaming, run details, and compact on-demand Chat information.
7. Preserve Files, Browser, and Terminal as direct brokered composer lanes; Reply returns to Chat semantics and restores the previous mode.
8. Verify long transcripts, attachments, hover/focus actions, reduced motion, responsive wrapping, and no document-level overflow.

## Verification Process

- Renderer and service integration tests for search, pending promotion, composer state, modes, model/provider navigation, atomic capability changes, attachment preflight, Reply, streaming, and provenance.
- Playwright r013 route/state walkthroughs for ordinary Chat, capability conflicts, search, all composer modes, responsive behavior, keyboard, console, and overflow.
- Native Light/Dark captures of representative conversation, model popover, conflict, pending Chat, search, and long-content states.

## Acceptance Criteria

The user reviews and accepts a production Chat walkthrough covering a normal turn, session search, provider/model browsing, compatible and incompatible attachments, reasoning-supported/unsupported models, all composer modes, Reply, and wide/narrow layouts.

## Implementation Notes

Started 2026-07-27. Task 00017's shared composer dock and shell geometry are in production. The corrective pass retained the Rust-owned Conversation, Artifact, Configuration, and picker services while correcting the following production ownership and r013 hierarchy gaps:

- project/session navigation now begins with the compact Search field and owns the Projects heading without an extra shell header; accepted icons replace placeholder glyphs while action names remain accessible;
- ordinary assistant identity now uses the C4OS mark plus the active model after the full-width Activity/Reasoning disclosure;
- the Chat toolbar now exposes all four approval presets and persists an exact selection through the native Configuration service without changing unrelated settings or bypassing managed policy ceilings;
- compatibility preflight offers Use compatible model, Convert, Remove file, and Cancel; because no approved converter is installed, Convert reports that bounded unavailability instead of simulating conversion;
- Cancel preserves the active model, draft, and attachments, then Send re-runs and re-exposes the unresolved preflight instead of leaving a silently disabled draft;
- Chat attachment conflicts no longer disable unrelated Files, Browser, or Terminal lanes; Reply keeps existing Chat attachments visible for the same preflight;
- Files Open invokes the native opaque-grant file picker. The renderer path field is read-only so a typed path cannot bypass native selection authority.

## Verification Notes

Component and service-integration verification passed on 2026-07-27:

- `npm run typecheck`
- `npm run lint`
- `npx vitest run tests/frontend/features/conversation tests/frontend/features/shell` — 19 files and 126 checks passed; jsdom emitted its known non-failing canvas notice.
- `npm run tauri:build` — debug application, production web bundle, sidecar source-pin verification, and `.app` bundling passed.
- The rebuilt production app launched through macOS accessibility as `com.c4os.desktop`; its ordinary user state correctly remained on the exceptional locked-Keychain recovery route. Task 00022 then completed the explicitly qualified QA-native Chat/Settings round trip, live Light/Dark transition, final wide/narrow conflict, model/provider, approval, information, Reply, mode, and lane captures, and 47/47 Playwright matrix without mistaking fixture evidence for persisted production provider state.

## Acceptance Notes

The integrated 472-test renderer, 47-test Playwright, complete Rust workspace, protocol, bundle, QA-boundary, and QA-native Chat/Settings-round-trip gates passed in Task 00022. Explicit user review remains pending.
