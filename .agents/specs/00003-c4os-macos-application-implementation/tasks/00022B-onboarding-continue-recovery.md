# Task 00022B — Onboarding Continue Recovery

Status: verified

Parent: [Task 00022](00022-r013-integrated-human-acceptance.md)

Coverage: corrective recovery handling for SET-001 and Task 00018.

## Summary

Resolve the user's Task 00022 finding that a successful OpenRouter Test Connection clears the visible API key, but Continue fails with `Workspace state is unavailable` and disables the only retry path.

## Implementation Steps

1. Repair the exact legacy comment-only `~/.c4os/config.toml` placeholder before strict configuration activation without accepting other invalid or externally changed documents.
2. Keep the Rust-owned successful transient test and credential available until provider, configuration, and credential persistence commit as one successful onboarding operation.
3. Remove a newly stored persistent credential on every pre-commit failure and serialize completion so the same transient token cannot race two commits.
4. Keep Continue enabled after a retryable completion error while preserving the failure live region and never restoring the raw key to renderer state.
5. Treat active-runtime route reload as post-commit reconciliation: failed peers remain quarantined, but already durable onboarding is not reported as an onboarding failure.

## Verification Process

- Focused Rust coverage for exact legacy-placeholder migration, same-token serialization, pre-commit token retention and credential cleanup, and ignored post-commit route-reload failure; focused React coverage for failed-Continue retry without another Test or raw-key entry.
- Full renderer, Playwright, Rust, protocol, production bundle, and diff validation.
- Rebuilt production-native walkthrough of Test Connection, Continue, Workspace Start, and restart read-back using the user's provider credential.

## Acceptance Criteria

After one successful Test Connection, the key field clears to its secure placeholder. If Continue encounters a retryable pre-commit failure, the error remains visible and Continue remains enabled. A retry succeeds without re-entering the API key, enters Workspace Start, and survives restart.

## Implementation Notes

Started 2026-07-28 from an explicit user finding during Task 00022 review. The durable app configuration had no last-known-good record while the editable file contained the exact legacy `# C4OS configuration` placeholder. Strict configuration correctly rejected that non-schema document, and the first onboarding save then detected the still-present file as an external replacement. The command also consumed its transient test before persistence and the renderer disabled Continue whenever an operation error was displayed.

The corrective implementation conditionally migrates only that exact legacy placeholder to the canonical schema document, leaves all other invalid or externally edited files fail-closed, serializes onboarding completion, retains the transient test until durable success, removes newly stored credentials on every pre-commit error, and keeps the successful test retryable in the renderer. A post-commit route reload still quarantines stale peers on failure but no longer strands first launch after the provider and configuration are already durable.

## Verification Notes

Two focused Rust migration regressions pass: the exact legacy placeholder is upgraded and a similar externally edited document remains untouched. Three production-wired onboarding settlement regressions directly prove same-token serialization, transient-token retention plus removal of the newly stored credential after an injected pre-commit failure, and successful durable completion despite an injected route-reload failure. The focused React regression proves that a failed Continue remains visible and retryable without another Test or raw-key entry. At the pre-native-handoff checkpoint, the renderer passed 80 files / 473 tests, protocol generation was consistent, Playwright passed 47/47 journeys, the Rust library passed 289 tests / 4 intentionally ignored, and the exact MCP STDIO fixture passed 2/2 outside the outer sandbox. All three packaged sidecar/resource validators passed; that checkpoint binary SHA-256 was `8a83f099364a27a7106623773e1305c148984d77a28c46ccdfcf3850be4d3c8e`. The final refreshed counts and bundle identity are recorded in the parent review package.

Two independent read-only follow-ups audited the production transaction seam and its concrete pending-token/transient-vault tests. Both returned P0=0, P1=0, and P2=0; no correctness, security, regression, or evidence-scope finding remains in this revision.

The rebuilt production app started against the real C4OS Home, conditionally migrated the 21-byte legacy placeholder to `schema_version = 1`, persisted generation 1 as the app-configuration LKG, and presented clean OpenRouter onboarding. The user then completed the production-native connection flow. The key cleared after Test, Continue entered Workspace Start, the OpenRouter profile and opaque credential survived the secure-storage recovery relaunch, and Settings read the provider back without the session-only warning.

The same live pass exposed two downstream runtime defects outside the original Continue transaction: the generated OpenCode provider configuration omitted the required public bootstrap that activates C4OS's private request credential channel, and upstream OpenCode `1.18.3` continued its session loop after a terminal provider `stop` response. C4OS now supplies only the fixed public bootstrap in configuration while the real key remains operation-scoped, and its reproducible downstream OpenCode build stops the turn when `finished` is true and no tool call remains. A fresh production Chat completed with one user message, one assistant message, one `step-finish: stop`, and no second native step.

The walkthrough also found that durable Retry/follow-up dispatch after app/runtime restart had no process-local native session correlation. Existing-chat dispatch now idempotently re-establishes the native session before broker activation while preserving each peer's correlation and route validation. A fresh-registry regression starts with no process-local session state and proves Retry creates that state before dispatch. The complete 26-test `runtime_dispatch` target passed, and Computer Use retried the previously failed durable Chat after a full rebuild/relaunch into a new native session. The new C4OS attempt completed in generation 2 with 32 durable events; its OpenCode database contains exactly one user message, one assistant message, and one terminal `stop` step.

## Acceptance Notes

Implementation, automated verification, production-native onboarding, restart read-back, fresh production Chat, and post-restart durable Retry are complete. Parent Task 00022 still requires the user's explicit integrated visual/functional acceptance.
