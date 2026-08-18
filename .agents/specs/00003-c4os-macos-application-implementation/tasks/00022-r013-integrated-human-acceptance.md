# Task 00022 — r013 Integrated Human Acceptance

Status: started

Coverage: corrective closeout for UX-002, UX-009, UI-004, UI-005, QA-001, QA-002, and every ID reopened by Tasks 00016 through 00021.

## Summary

Run the complete production-rendered r013 convergence matrix and obtain explicit human acceptance. Automated correctness, security, accessibility, and containment remain validation; they do not substitute for review of the visible product journey, geometry, density, content hierarchy, and interaction behavior.

## Implementation Steps

1. Build a deterministic review matrix for all 16 accepted destinations and every material state changed by Tasks 00016 through 00021 without exposing fixture controls in production.
2. Capture production and r013 reference views at matched wide and narrow dimensions, with Light/Dark pairs where platform adaptation matters.
3. Record the normal first-run journey, returning-user journey, Settings round trip, ordinary/capability-aware Chat, search, all composer modes, and every artifact focus/restore path.
4. Run the complete security, credential, provider/runtime, persistence, accessibility, keyboard, native, responsive, reduced-motion, console, overflow, restart, failure, and recovery matrices.
5. Present a concise comparison package that distinguishes exact r013 parity, platform-native adaptation, real-service additions, and any remaining intentional difference.
6. Convert every user-rejected or unresolved visible difference into a new open corrective task; do not mark this task accepted while review findings remain.
7. After explicit acceptance, close the corrective coverage supplement and review implementation-discovered facts for Context promotion without changing the Frozen contract retroactively.

## Verification Process

- Full frontend, Playwright, Rust, protocol, bundle, native, security, accessibility, responsive, recovery, secret/log/process, console, and Agent Workspace validation gates.
- Exact evidence index with commands, dimensions, hashes, source revision, limitations, and production-versus-QA qualification.
- Read-only independent review of the final diff and launched application before the user walkthrough.

## Acceptance Criteria

The user explicitly accepts the complete production-rendered r013 comparison package and practical user journeys. This task cannot finish at `verified`; it reaches `accepted` only after the user's visual and functional review.

## Implementation Notes

Started 2026-07-27 after Tasks 00016 through 00021 reached their implementation and component/authority verification checkpoints. The user's corrective-goal instruction explicitly authorizes continuous execution without intermediate human acceptance, so this final integrated pass is also the first consolidated review package for those six visible tasks. Their `accepted` states and corrective closeout remain gated on the user's review of this package.

The final pass refreshed the deterministic provider Test/Continue adapter so Test remains transient and Continue persists the confirmed provider/model/defaults once; added QA-only failed and zero-usable-model projections; repaired stale route and heading selectors; replaced the last visible placeholder attachment glyph with the shared File icon; removed duplicated Providers/MCP route headings; and reconciled an older Rust approval-generation test with the accepted direct-intent contract by installing an explicit `Ask` rule. No production authority was transferred to the renderer or QA adapter.

The first independent closeout audit found two P1 issues: the Providers surface still repeated its route title inside the page component, and the evidence package claimed a broader freshly captured material-state matrix than it indexed. The corrective follow-up removed the component-owned Providers title/support, added a title-ownership regression, captured every Settings destination wide/narrow plus the shared dialog, added the complete onboarding exception/success matrix and first-run video, added Chat mode/popover and Browser/File/Folder representatives, captured native Light/Dark pairs, and explicitly layers unchanged provider-specific native evidence through Tasks 00008–00010. The package no longer claims that every provider-specific state was freshly recaptured.

The audit follow-up found one further P1 acceptance-evidence gap: Task 00016 also names an explicit-`Ask` provider action and a runtime-initiated credential action. Build-gated QA adapters now exercise both policy paths. The 1100x761 native runtime capture mounts the production approval center, while Playwright independently verifies the same decisions through QA-composed approval panels. Together the captures prove pause-before-effect/no-persistence for Provider Test and an explicit credential boundary for the runtime action. The QA surfaces are visibly qualified, and the final production bundle excludes their authority markers.

The exact evidence index, qualifications, comparison matrix, commands, dimensions, hashes, and limitations are recorded in the [Task 00022 review package](../../../resources/native/task-00022-review-package.md).

## Verification Notes

Integrated verification completed 2026-07-27:

- `npm run qa:renderer`: formatting, lint, typecheck, 80 Vitest files / 472 tests, production and QA builds, and both QA-boundary checks passed;
- `npm run test:app`: 47/47 Playwright tests passed across all accepted routes, explicit-`Ask` Provider Test, runtime credential approval, responsive widths, Settings round trip, Chat/capabilities/search, File/Folder/Browser focus, Terminal lifecycle, reduced motion, console, and overflow;
- `cargo test --workspace --no-fail-fast -- --test-threads=1`: every non-ignored workspace test passed on the host tier, including the pinned-Node MCP STDIO fixture; the library tier reported 284 passed / 4 intentionally ignored;
- `npm run protocol:check` and all three final `.app` sidecar/resource verification commands passed;
- the Agent Workspace validator passed with two pre-existing preferred-line-count warnings and no hard errors;
- headed browser inspection covered 1440x900, 700x840, and 390x844 product surfaces, with zero console errors/warnings in the final product session;
- Computer Use inspected QA-native Workspace Start, Chat, Settings, live Light-to-Dark adaptation, exact Back restoration, explicit-`Ask` Provider Test, and runtime credential approval at 1100x761, then inspected the final production `.app` without a QA marker and verified truthful locked-Keychain Retry behavior in Light and Dark; system appearance was restored to Dark;
- after the login Keychain was unlocked, Computer Use exercised a fresh production-native disposable namespace: one Test and one Continue succeeded without a C4OS approval, restart reopened Workspace Start, and Settings read back the persisted provider; non-secret Keychain metadata and a plaintext scan verified the storage boundary before exact cleanup;
- final production binary SHA-256 is `7e58420feb68e3221cd8b4d9d91af830d7e014ce0c303266f91adc5f72e46ef3`; C4OS-owned processes were terminated after review.
- the pre-Task-00022B independent read-only closeout verdict was P0=0, P1=0, and P2=0 after the package distinguished the native production approval center from the QA-composed browser companion.

The unlocked native rerun exposed and repaired two further P1 generation defects before evidence was accepted: stale pre-effect Provider Test now refreshes and retries once without replaying a network effect, and mediated Continue now commits with the post-authorization coordinator generation. The focused frontend/Rust regressions, 472-test renderer suite, 47-test Playwright suite, complete host-level Rust workspace run, protocol check, rebuilt production `.app`, and bundle validators all pass after those repairs.

User review then identified one visible contract error: successful onboarding showed a model picker and defaults panel absent from r013. Required side quest [Task 00022A](00022A-onboarding-automatic-model-selection.md) reopened Task 00018, removed that extra step, and made the Provider service automatically choose the viable model with the most normalized C4OS-supported features. The correction and refreshed evidence are verified; Task 00022 remains started until the user accepts the corrected package.

The refreshed corrective pass again passed 80 Vitest files / 472 tests, 47/47 Playwright tests, the complete non-ignored Rust workspace with 284 library tests / 4 intentionally ignored, protocol consistency, the rebuilt production `.app`, all bundle/resource validators, and the production QA-marker scan. The final binary SHA-256 is `7e58420feb68e3221cd8b4d9d91af830d7e014ce0c303266f91adc5f72e46ef3`. Wide and narrow successful-Test captures and the complete first-run recording now contain no model picker or defaults panel.

The user's next production retry exposed another blocking first-run defect: the visible API key correctly cleared after Test, but Continue failed with `Workspace state is unavailable` and disabled retry. Required side quest [Task 00022B](00022B-onboarding-continue-recovery.md) traced the failure to the exact legacy comment-only app configuration placeholder, repaired its conditional migration, retained the Rust-owned transient test until durable success, added complete credential cleanup for pre-commit errors, and kept Continue enabled after retryable completion failures. Focused migration, backend settlement, and renderer-retry regressions passed, including direct injected coverage for same-token serialization, pre-commit token/credential recovery, and ignored post-commit route-reload failure. At that checkpoint the renderer, Playwright, protocol, prior complete Rust workspace, packaged-sidecar/resource, and production-build gates passed; the Rust library tier reported 289 passed / 4 intentionally ignored, two independent read-only follow-ups returned P0=0/P1=0/P2=0, and the checkpoint binary SHA-256 was `8a83f099364a27a7106623773e1305c148984d77a28c46ccdfcf3850be4d3c8e`. The later native handoff and its additional findings are recorded below.

The user completed that native handoff on 2026-07-28. Production Test/Continue passed, secure-storage recovery relaunched into installation-key protection, and the saved provider read back after restart. The first real Chat then exposed a native integration chain that deterministic provider tests could not: C4OS needed a non-secret OpenCode bootstrap to activate its private request credential channel, and pinned OpenCode `1.18.3` continued after a terminal provider `stop`. Both are repaired without moving the real key into configuration. The reproducible downstream build `c4os-auth-fd-terminal-stop.2` uses patch SHA-256 `512a8a82a157b5e7952c9637b6d4a7a03535418b868f8d25b9a06540c6f6a3b1` and native binary SHA-256 `99d5d922f715ea0df9605f72849c68c0404d48b43ff319cef4cf943a0ee650e8`. A fresh native Chat completed in three seconds with exactly one user message, one assistant message, and one `step-finish: stop`.

One final restart-recovery audit found that durable Retry/follow-up dispatch did not recreate process-local native session correlation after app/runtime restart. The dispatch registry now idempotently establishes the native session before an existing-chat dispatch while preserving each peer's correlation and route validation. A fresh-registry regression begins with empty process-local session state and proves creation precedes dispatch; the complete 26-test dispatch target passed. After a full rebuild/relaunch, Computer Use retried the earlier failed durable Chat into a new OpenCode session; its new attempt completed in generation 2 with 32 durable events, and the native database again contains exactly two messages and one terminal `stop` step. The final production binary SHA-256 is `0461a21b1c7cfc489bca980106c453548e36e8e2514909906208695f6d69897f`.

Tasks 00016 through 00021 are now `verified`. Task 00022 remains `started` because its contract requires explicit user acceptance rather than a coordinator-assigned `verified` state.

## Acceptance Notes

The no-picker and Continue-recovery findings are implemented and verified. Production-native onboarding, secure-storage restart read-back, a bounded fresh Chat, and durable post-restart Retry now pass in the rebuilt app. No task is marked accepted by this record; the remaining gate is the user's explicit acceptance of the complete visible and functional result.
