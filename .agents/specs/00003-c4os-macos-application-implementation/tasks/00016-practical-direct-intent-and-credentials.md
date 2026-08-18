# Task 00016 — Practical Direct Intent And Credentials

Status: verified

Coverage: corrective support for UX-010, UX-015, SET-001, SET-004, and SET-005.

## Summary

Make direct user actions practical without weakening Rust-owned credentials, the Action Gateway, sandbox ceilings, trusted roots, managed policy, or denial-before-effect. Remove duplicate C4OS consent prompts from exact user-initiated provider operations, repair the normal macOS credential path, and keep explicit prompts for delegated, changed, policy-selected, destructive, or otherwise higher-risk actions.

## Implementation Steps

1. Preserve the native Keychain OSStatus and operation context instead of collapsing every failure into `KeychainUnavailable`; add a bounded retry path and prove the configured application/service identity used by the local macOS build.
2. Define one direct-intent decision table for exact user controls. A direct click grants the named action and target without a second C4OS prompt unless an explicit `Ask` rule applies, the target/version changes, authority expands, the action is destructive or irreversible, credentials would leave the declared provider boundary, or managed/platform policy requires interaction.
3. Keep runtime-, agent-, plugin-, MCP-, and background-initiated actions on the existing policy and approval path. Preserve denial-before-effect, single-use authorization, serialization, expiry, audit, and redaction.
4. Change provider Test Connection to use the submitted credential transiently through the Rust boundary without first persisting the profile or secret.
5. Persist the provider and opaque credential reference once, when Continue confirms the successful current test and proposed model/runtime/environment defaults.
6. Treat session-only credentials as an exceptional degraded choice. Explain volatility, offer Retry secure storage, never imply an Apple permission prompt occurred, and never silently fall back to plaintext or durable renderer state.
7. Apply the same no-duplicate-prompt rule to exact direct Settings actions such as Refresh, Save, enable/disable, and Test while retaining explicit confirmation for destructive removal where the accepted interaction requires it.

## Verification Process

- Rust policy and credential tests for direct intent, explicit `Ask`, managed deny, changed targets, credential egress, runtime-initiated actions, denial-before-effect, audit, expiry, and redaction.
- Native macOS tests for successful Keychain create/read across restart, preserved OSStatus on failure, Retry, and session-only fallback.
- Provider integration tests proving Test does not persist a profile/key, Continue persists exactly once, and no secret reaches renderer snapshots, logs, arguments, configuration, or Workspace files.
- Production-rendered walkthroughs that count prompts on the normal path and verify exceptional prompts remain truthful and actionable.

## Acceptance Criteria

The user reviews native recordings or captures of the normal secure-storage path, the Keychain-unavailable recovery path, an explicit-`Ask` provider action, and a runtime-initiated credential action. The normal path must contain no C4OS `Allow once` prompt after the user directly activates Test or Continue.

## Implementation Notes

Started 2026-07-27. The initial production-native onboarding inspection confirmed two current-path failures before implementation: the application enters the exceptional session-only credential state on an ordinary local launch, and an untested provider form renders a large stale disabled model list. This task preserves the security architecture while correcting those failures and changing when product-level consent is redundant.

## Verification Notes

Automated implementation verification passed 2026-07-27:

- focused Rust Action Gateway, credential-vault, policy-authorization, and provider-lifecycle suites: 84 passed, one explicitly opt-in live Keychain tier ignored;
- frontend typecheck plus focused Provider onboarding, Settings, controller, and adapter suites: 24 passed;
- debug acceptance helpers and configured Keychain service-identity assertions: 4 passed;
- non-QA production web build, sidecar verification, and exact debug `.app` bundle completed successfully before the first corrected native capture.

The rebuilt native onboarding capture proves the exceptional recovery path now preserves the actual `ReadExistingInstallationKey` failure, exposes `Retry secure storage`, offers session-only storage only as an explicit degraded choice, hides the stale pre-Test model catalog, and never claims an Apple permission prompt occurred. On this host the login Keychain returns OSStatus `-25293` for both the existing production service and a unique disposable `dev.c4os.live-test.*` create attempt; `security show-keychain-info` independently reports that the login Keychain cannot authenticate. The existing production-looking Keychain item and real `~/.c4os` vault were not modified, and the failed disposable attempt left no item behind.

After the user unlocked the login Keychain, the integrated Task 00022 pass exercised that isolation boundary with a fresh mode-0700 acceptance home, a unique `dev.c4os.live-test.*` Keychain service, and a localhost-only OpenAI-compatible HTTP fixture. The first native attempt exposed two additional P1 generation defects: Test required a redundant explicit retry after a harmless coordinator advance, and mediated Continue compared its pre-effect generation after the Action Gateway had advanced it. The frontend now retries only the pre-effect stale Test/Save cases once after refreshing its private cursors, and the Rust completion path uses the post-authorization coordinator generation only for mediated effects. Focused regressions cover both rules.

The rebuilt non-QA `.app` then completed the normal path with one Test click and one Continue click, no C4OS `Allow once` prompt, one production-ready model, and direct entry to Workspace Start. Non-secret Keychain metadata confirmed creation of the disposable installation-key item. The isolated vault, configuration, and SQLite state contained no plaintext submitted credential; their captured pre-cleanup SHA-256 values were `fd2146a311d1ac2d73b3882eeeed15bdaa60f0769d49aa68ecbc7abccfc1f0da`, `c338bd4321ea86431458b3ca4747b4e886cae37670d09f08f21ea5b44ba4b059`, and `da63a52bdf62b3fc863995d79f497584be3ba97c1de4c55d1d301afbafab6cfc`. A controlled restart reopened Workspace Start, and native Settings projected the persisted `Native Keychain Acceptance` provider. The exact disposable Keychain item, isolated homes, local fixture, and C4OS process were removed afterward; the real C4OS installation key and `~/.c4os` remained untouched.

The final corrective follow-up also captured the other two native acceptance paths at 1100x761. The explicit-`Ask` Provider Test used a disposable QA credential and paused before the provider effect or profile persistence, with the secure field cleared and Deny/Allow once visible. The runtime-initiated native path mounted the production approval center over the qualified QA runtime surface and explicitly stated that OpenCode requests temporary OpenAI credential use while credential bytes remain inside the provider boundary and never enter renderer state or runtime arguments. Playwright separately verifies the same policy decisions through the QA-composed approval panels; both native captures remain visibly labeled as deterministic QA state. The production bundle excludes those QA authorities and labels.

## Acceptance Notes

Implementation and verification are complete for the normal secure-storage create/read/restart path, exceptional Keychain recovery, explicit-`Ask` Provider action, and runtime-initiated credential action. Task 00016 is verified and awaits only the user's explicit review of the integrated Task 00022 package; it is not marked accepted by this record.
