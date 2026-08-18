# Task 00022A — Onboarding Automatic Model Selection

Status: verified

Parent: [Task 00022](00022-r013-integrated-human-acceptance.md)

Coverage: corrective rejection handling for SET-001 and Task 00018.

## Summary

Resolve the user's Task 00022 finding that production onboarding inserted a model picker absent from r013. Keep the compact provider form and let the Rust-owned Provider service automatically choose the viable model with the broadest C4OS-supported feature set.

## Implementation Steps

1. Remove the post-Test model picker and default-confirmation panel from onboarding without removing Models Settings or ordinary post-onboarding model controls.
2. Rank production-ready models by the number of normalized capability features in the explicit `supported` state, then by discovery recommendation rank and stable model identity.
3. Preserve a valid explicit model selection during later provider retests, but use the automatic ranking for a new provider and whenever no valid selection remains.
4. Continue with the automatically selected model through the existing transient-Test and one-time credential-safe persistence boundary.
5. Correct the drifted Context, launch/settings reference, Frozen contract erratum, Task 00018 record, and Task 00022 review package.

## Verification Process

- Focused React tests prove successful Test shows only the compact status, no picker/default panel, and Continue submits the strongest supported model exactly once.
- Focused Rust tests prove an available model with more supported features wins over earlier catalog order or an upstream recommendation, while unavailable models cannot win and later explicit selection remains possible.
- Playwright re-exercises onboarding with multiple production-ready models at wide and narrow widths and refreshes the successful-state evidence.
- Full renderer, Playwright, Rust, protocol, production bundle, evidence-hash, Agent Workspace, and diff checks run before handoff.

## Acceptance Criteria

The user reviews the refreshed successful-Test onboarding capture and confirms there is no model picker or default-confirmation step. Continue remains enabled, and the strongest viable model is selected internally for the next state.

## Implementation Notes

Started 2026-07-27 from an explicit user finding during Task 00022 review. This side quest is required because the accepted r013 artifact contains only provider fields, the status live region, Test Connection, and Continue; the prior KB/task wording had drifted beyond that visible contract.

The shared provider form no longer imports or renders onboarding model choices or a default-confirmation panel. Its obsolete presentation styles were removed. The frontend fallback and Rust-owned Provider service now rank only production-ready models by descending normalized `supported` feature count, then ascending discovery rank, then stable model identity. A still-valid explicit selection survives later provider retests; a new provider or invalidated selection uses the automatic ranking. Continue carries that internal result through the existing transient-Test and one-time credential-safe persistence boundary. Models Settings and Chat model controls remain available after onboarding.

The Context, launch/settings reference, Frozen-contract erratum, coverage ledger, Task 00018 record, QA adapter, and Task 00022 evidence package now express the same no-picker contract.

## Verification Notes

Verification completed 2026-07-27:

- 25 focused React tests passed across Provider onboarding, Provider Settings, and the production-route QA adapter. The success case exposes only `2 production-ready models discovered.`, proves the picker/default copy is absent, and submits the stronger `gpt-5` route even though the less capable model appears first with the earlier discovery rank.
- The focused Rust lifecycle regression passed, proving normalized supported-feature count outranks catalog order and upstream recommendation, unavailable models cannot win, and a later explicit selection remains possible.
- `npm run qa:renderer` passed formatting, lint, typecheck, 80 Vitest files / 472 tests, production and QA builds, and both QA-boundary checks.
- `npm run test:app` passed 47/47 Playwright tests after the default sandbox attempt failed only because that sandbox denied the local Vite listener.
- `npm run qa:rust` passed every non-ignored workspace test; the library tier reported 284 passed / 4 intentionally ignored, and provider lifecycle passed 25/25.
- Protocol generation/check, production `.app` build, production bundle, all three bundled-sidecar/resource checks, production QA-marker scan, Agent Workspace validation, manifest hashes, and `git diff --check` passed.
- Refreshed 1440x900 and 390x844 successful-Test captures show the compact success live region and Continue with no picker/default panel. The refreshed 1440x900 recording continues from blank onboarding through Test, Continue, Workspace Start, and Chat.

## Acceptance Notes

Implementation and verification are complete. Explicit user review of the corrected capture remains pending; this task is not marked accepted by this record.
