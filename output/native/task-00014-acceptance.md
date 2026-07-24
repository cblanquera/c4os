# Task 00014 Updates, Recovery, Diagnostics, And Degraded-State Acceptance Evidence

Date: 2026-07-24

Verified predecessor checkpoint: `6f6672d`

Final rebuilt binary:

- Path: `target/debug/bundle/macos/C4OS.app/Contents/MacOS/c4os`
- SHA-256: `755376d2be6381f6b3058081c626ac6350d8f3d13057c0bea4d96643a7f59872`
- Binary timestamp: 2026-07-24 13:26:17 PST, after the final `core/services.rs` repair at 13:18:36 PST

Disposable native homes:

- Healthy: `/private/tmp/c4os-task14-final-healthy-home`
- Degraded: `/private/tmp/c4os-task14-final-degraded-used2`
- The user's original `~/.c4os` was moved to a private temporary backup only while each acceptance home was active, then restored at its original `0755` mode. Both acceptance homes returned to owner-only `0700` mode.

## Accepted production behavior

1. Application, OpenCode, Pi, and Plugin lifecycle state remains independently versioned and Rust-owned. Local candidates are verified and staged without claiming signing, notarization, publication, or distribution.
2. Application candidate `0.2.0` remains staged beside current `0.1.0`. Its failed compatibility result is `application_rebuild_required`, so the renderer exposes Rebuild Application and Revoke staged update but no Activate action.
3. Interrupted operations, recovery notices, last-known-good state, revocation, rollback, cleanup, and diagnostics remain durable and generation checked.
4. Startup boundary failure blocks Settings, Workspaces, Chats, and every product route. Retry relaunches the app and returns to the blocked recovery card with a fresh correlation; no Continue path is exposed.
5. App- and Workspace-scoped policy recovery is sticky. Ordinary errors, rejected content, no-op/unchanged reconciliation, and self-write deduplication cannot clear unresolved recovery. Only proven compensation or successful observer reconciliation of current durable last-known-good authority may clear it.
6. Persistent stable content rejection settles after watcher coverage instead of retrying forever. Configuration diagnostics are exactly deduplicated and bounded to the newest 128 records.
7. Diagnostics expose only bounded category, severity, component boundary, message, correlation, recovery action, and timestamp fields. Prepared export metadata is path-free and does not expose candidate identifiers, artifact digests, credentials, prompts, environment values, or raw provenance.

## Renderer quality matrix

| Gate | Result | Measured time |
| --- | --- | ---: |
| `npm run format:check` | passed | 3.15 s |
| `npm run lint` | passed with zero errors | 6.51 s |
| `npm run typecheck` | passed | 4.29 s |
| `npm test` | 77 files and 441/441 tests passed | 12.47 s |
| production web build | passed | 5.13 s |
| QA web build | passed | 5.02 s |
| `npm run test:e2e -- --workers=1` | 43/43 passed | 26.24 s wall; 25.5 s suite |

The complete renderer and Playwright matrices ran after the update-action repair. The already-passing focused Terminal race and four QA route/Settings cases from the earlier 37/42 handoff were not repeated unchanged.

## Rust, protocol, bundle, and native tiers

| Gate | Final result | Measured time |
| --- | --- | ---: |
| `cargo test --workspace --no-fail-fast` | every runnable library, integration, and doc-test target passed; the restricted run's only failures were the two known MCP STDIO child-process cases | 2234.61 s wall |
| exact host `mcp_transport_fixtures` rerun | 2/2 STDIO tests passed; two HTTP tests remained explicitly ignored in this tier | 4.14 s wall |
| final configuration-focused library slice | 10/10 passed | 2.88 s wall after incremental compile |
| Workspace sticky content-rejection regression | 1/1 passed | 148.20 s including compile |
| app sticky recovery slice | 3/3 passed, including two new regressions | 59.14 s |
| `cargo check --workspace` | passed; only the existing `ts-rs` unsupported serde-attribute notices remained | 146.49 s |
| exact `protocol_contract export_bindings` | 1/1 passed | 23.13 s |
| final `npm run tauri:build` | passed and copied the pinned OpenCode/Pi resources | 387.91 s |
| `npm run bundle:opencode-sdk:verify` | 1/1 passed | 156.08 s wall; 1.31 s test |
| `npm run bundle:opencode-assets:verify` | 1/1 passed | 60.14 s wall; 6.91 s test |
| `npm run bundle:pi:verify` | 1/1 passed | 71.48 s wall; 12.11 s test |
| ignored MCP Streamable HTTP tier | 2/2 passed against the final source | 34.48 s wall; 3.07 s test |
| ignored OpenCode bundle/native/loopback tier | 7/7 passed against the final bundle | 25.46 s wall; 23.21 s test |
| ignored OpenCode streaming tier | 3/3 passed | 36.42 s wall; 0.05 s test |
| ignored packaged production-runtime tier | 4/4 passed against the final bundle | 151.59 s wall; 119.61 s test |
| `tools/run-task-00004-native-golden.zsh` | private-TLS OpenCode/Pi golden passed 1/1 against the final bundle | 60.55 s wall; 57.94 s test |

The broad `npm run protocol:generate` name filter and one broad focused Cargo name filter were stopped after their desired library tests passed but before they redundantly enumerated every unrelated integration target. Neither stopped command is counted as acceptance. The exact protocol target and exact focused library targets passed.

## Final native macOS matrix

| Check | Result |
| --- | --- |
| Healthy launch | passed; the rebuilt bundle restored the production Workspace Start route from the disposable healthy home |
| Update snapshot | passed; generation 90 showed current app `0.1.0`, staged candidate `0.2.0`, `Recovery required`, `application_rebuild_required`, Rebuild Application, and Revoke staged update |
| Unsafe activation suppression | passed; `Activate staged update` was absent from the accessibility tree and visible card because recovery action was `rebuild_application` |
| Diagnostics export | passed; export `diagnostics-a62bab3f9146aea7b05a9269` contained 85 records with digest `sha256:a62bab3f9146aea7b05a926961aa352e40005e32c9894f6cc61cf339b1272087` and no path |
| Narrow layout | passed at 624×761; update recovery and Diagnostics remained contained with no document-level horizontal overflow |
| Controlled restart | passed; native Quit left no app/sidecar process, relaunch returned to Start, and Settings retained the same staged recovery at generation 99 with no Activate action |
| Degraded startup | passed; a directory at `state/app.sqlite3` produced `startup-boundary-unavailable`, blocked every product route, and exposed only Retry and Refresh |
| Degraded retry | passed; retry relaunched into the same blocked card with correlation `startup:1784871561348`, one bounded history item, and no Continue action |
| Home restoration | passed; original `~/.c4os` was restored at `0755`; both disposable homes returned to `/private/tmp` at `0700` |
| Final cleanup | passed; no C4OS, OpenCode, Pi, MCP fixture, or private-TLS fixture process remained |

The final healthy database retained update generation 99, a 29,720-byte canonical update document, 94 structured diagnostics, no pending operations, and no recovery notices. The diagnostic count remains below the fixed 128-record bound.

## Redaction, persistence, and log inspection

- The 94 update diagnostic records use exactly these keys: `diagnosticId`, `correlationId`, `category`, `severity`, `componentBoundary`, `message`, `recoveryAction`, and `createdAtMs`.
- Maximum diagnostic message length was 61 bytes.
- Unsafe diagnostic shapes matched zero records: no raw `/Users` or `/private` paths, `file://` values, bearer forms, API-key/password/secret/token assignments, path fields, candidate identifiers, or artifact digests.
- Binary-safe scans across both acceptance homes found zero credential-shaped assignments, bearer values, raw local paths, or file URLs.
- The final 30-minute unified-log scan covered 9,111 C4OS lines and found zero exact unsafe path, candidate, digest, bearer, secret, password, or token-assignment matches.
- The 29 generic `token`/`authorization` lines were emitted only by Apple `AppSSO`, `AppSSOCore`, `WebKit`, `LaunchServices`, and `libxpc` senders.

## Evidence images

| File | Purpose | Dimensions | SHA-256 |
| --- | --- | --- | --- |
| `task-00014-final-update-staged.jpg` | final staged application recovery with no unsafe Activate action | 1100×761 | `10a2adf097ffcd623aced0cae0f35714ed19d7323ee55132e5f4ffe08cba6f65` |
| `task-00014-final-diagnostics-export.jpg` | prepared redacted, path-free diagnostic export | 1100×761 | `b21a65c693061df4249128d9d9adaa0457dec9f4e90fef767f2961381d485466` |
| `task-00014-final-narrow-update-diagnostics.jpg` | narrow update recovery and Diagnostics composition | 624×761 | `6ee3f6a1eaf29c109def1257b8cccd9ea66c97319978713c8efa3deaeaa7f6b9` |
| `task-00014-final-restart-persistence.jpg` | staged recovery retained after controlled native restart | 1100×761 | `a5f136dcb02af122bb06946571319f9dbe33e4aa3941c841d9dde67741b813f2` |
| `task-00014-final-startup-recovery-degraded.jpg` | startup-boundary failure, retry, blocked routes, and no Continue | 1100×761 | `0776384b8565d133352b6c2269a0b5215a8ccef1f30519e51ef8482c6a7723e4` |

## Independent Agent Acceptance

All final reviews were read-only and passed the required threshold:

- Rust/security: P0 = 0, P1 = 0, P2 = 1, P3 = 0.
- Renderer/integration: P0 = 0, P1 = 0, P2 = 1, P3 = 0.
- Policy/configuration: P0 = 0, P1 = 0, P2 = 1 after reciprocal coverage alignment, P3 = 0.

Retained non-blocking P2s:

1. `RestoreValidatedBackup` is effectively unreachable through normal production bootstrap because backup authority is captured only after successful Database startup. A future repair must preserve exact-Database scope and independently persisted, revalidated authority.
2. Pending update operations require manual Refresh because the renderer controller has no bounded polling or native subscription.
3. A successful Settings-based policy save can leave an older sticky app recovery notice visible until observer reconciliation, restart, or another externally activated change. The authority stays fail-closed; only the projection can remain conservatively degraded.

## Explicit external gates

This task does not sign, notarize, publish, deploy, distribute, or claim public updater readiness. Signed feeds, signing-key custody, notarization, release distribution, and human acceptance remain outside this local-development checkpoint and are routed to the later release/closeout work.
