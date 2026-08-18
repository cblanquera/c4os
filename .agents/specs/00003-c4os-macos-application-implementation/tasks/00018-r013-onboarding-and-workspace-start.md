# Task 00018 — r013 Onboarding And Workspace Start

Status: verified

Coverage: corrective ownership for SET-001 and SET-002; support for UI-004 and UX-012.

## Summary

Implement the practical first-run journey and Workspace Start composition against r013. Keep the real provider, credential, model, runtime, and Workspace services while presenting the compact workflow and state order accepted in the wireframe and Context contracts.

## Implementation Steps

1. Compose first-provider onboarding as the centered compact provider card with one-column field flow, conditional compatible-provider fields, Test Connection, and Continue; no Settings navigation or implementation commentary.
2. Keep Test and Continue immediately reachable. Show status in the form live region rather than inserting a permanent default status card.
3. Never render model choices in onboarding. After a successful exact-form test, automatically select the production-ready model with the most normalized `supported` features; use discovery rank and stable model identity as tie-breakers.
4. Keep the successful state inside the existing live region. Continue persists the automatic model with OpenCode and Local, while ordinary post-onboarding model controls own later changes.
5. Use Task 00016's transient Test and one-time Continue persistence. Surface exceptional secure-storage recovery without turning it into the normal journey.
6. Restore the r013 Workspace Start hierarchy: compact header/branding, exactly three primary action cards, recent Workspace list, progress/error states, and same-document transition to Chat.
7. Preserve native folder/archive selection, clone behavior, restart recovery, keyboard use, accessibility, and responsive stacking.

## Verification Process

- Provider field/state tests for presets, compatible fields, validation, fresh-test invalidation, zero/one/many models, automatic supported-feature ranking, absence of a picker, and exact persistence timing.
- Workspace Start tests for all three actions, recents, progress, error/recovery, same-document navigation, and restart.
- Playwright and native walkthroughs for normal, failure, degraded, success, submitted, returning-user, wide, and narrow states.
- Visual comparison against r013 onboarding and Start at equivalent viewports.

## Acceptance Criteria

The user reviews and accepts the complete first-run recording from blank launch through provider test, automatic model selection without another step, Continue, Workspace Start, and entry to Chat, plus separate failed-test, no-model, secure-storage-recovery, returning-user, and narrow-layout captures.

## Implementation Notes

Started after the Task 00016 provider/credential convergence and Task 00017 shared geometry foundation. The corrective implementation is limited to the onboarding and Workspace Start presentation while retaining the native Provider, credential, picker, clone, recovery, and navigation services.

- First-provider onboarding is now a centered 480px product-branded card. Provider fields are one column, and the live connection state, Test Connection, and submit-type Continue control share the same form.
- The ordinary untested state no longer renders as a permanent bordered status card. A 2026-07-27 user review then identified the successful-state model/default confirmation as wireframe drift; Task 00022A removed that panel and retained only the live-region result before Continue.
- Workspace Start now uses the r013 product bar, compact Workspace hierarchy, 820px content cap, exactly three action cards, at most three recent rows, and the accepted 760px icon-plus-two-copy-row stack.
- Existing native folder/archive selection, clone approval, restart recovery, focus restoration, and same-document Chat navigation were preserved.

## Verification Notes

Automated component verification passed on 2026-07-27:

- `npm run typecheck -- --pretty false`
- `npm run lint -- --max-warnings=0`
- 37 focused Vitest checks across Provider onboarding, Workspace Start screen/route, Launch routing, and the new r013 launch-surface style contract.

Task 00022A then refreshed the successful-Test wide and narrow captures and the complete first-run recording after removing the model/default panel. Focused provider verification passed 25 React tests and the Rust strongest-model regression. The final `npm run qa:renderer` gate passed 80 Vitest files / 472 tests, `npm run test:app` passed 47/47 Playwright tests, and the complete Rust workspace passed with 284 library tests / 4 intentionally ignored. Protocol, production build, bundle, Agent Workspace, evidence-hash, and diff gates also passed; see `.agents/resources/native/task-00022-review-package.md`.

## Acceptance Notes

The no-picker correction and refreshed successful-state evidence are verified. Explicit user acceptance of the corrected first-run capture and journey remains pending through Task 00022.
