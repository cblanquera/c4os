# Task 00015 Integrated Acceptance Evidence

Date: 2026-07-24

## Final build identity

- Branch: `build/mvp-3`
- Parent checkpoint: `56bd8d2`
- Build: local unsigned debug macOS application; no signing, notarization, distribution, publishing, or deployment
- Application: `target/debug/bundle/macos/C4OS.app`
- Binary SHA-256: `0fabf1fdfd4a9a63f2952a1aec7ab6dd89f935386ac32a3e37f7d41b5ed47a53`
- Binary size: `118382520` bytes
- Binary mtime: `2026-07-24 20:24:49 PST`
- Production frontend: `index-DqgUqPqV.js` and `index-ClYtt0KM.css`
- Production frontend source maps: zero
- Packaged QA markers (`VITE_C4OS_QA`, `deterministic-e2e`, `/qa/foundation`): zero

The production Resources tree contains dependency-owned sidecar maps. The enforced boundary is the packaged frontend asset graph; no frontend JavaScript map is embedded.

## Deterministic renderer and browser gates

| Gate | Result |
| --- | --- |
| `npm run qa:renderer` | passed: formatting, lint, typecheck, 79 files and 454 unit tests, production build, QA build, and both bundle-boundary checks |
| `npm run qa:playwright` | passed: 45/45 with one worker in 28.9 seconds |
| QA schema | fixed version, IDs, clock, adapter state, reset, replay, isolation, and all 16 accepted direct routes |
| Production/QA composition | production uses only product router/store/native transport; QA composition is selected only by the build-gated entry |
| Native QA transport | requires the explicit in-page `deterministic-e2e` harness marker and never falls through to real native authority |
| Production bundle | zero fixture routes, fixture marker strings, fixture source maps, or QA transport selection |

The 16-route native QA walkthrough passed route identity, fixture identity, wide and 626 px narrow geometry, keyboard navigation, native Settings entry, Back, Light/Dark composition, and direct-route isolation. Earlier capability screenshots remain evidence for theme and native chrome, not exact final route content.

## Rust, security, protocol, and bundle gates

| Gate | Result | Measured portion |
| --- | --- | ---: |
| saturated diagnostic regression | 1/1 | 36.48 s compile; 1.50 s test |
| maximal diagnostic replacement regression | 1/1 | 28.81 s compile; 0.03 s test |
| `update_lifecycle` | 15/15 | 1.93 s test |
| `database_persistence` | 37/37 | 0.99 s test |
| `npm run qa:rust` | all product library, integration, and doc-test targets passed; only two outer-sandbox MCP STDIO cases failed | 86 s build; 61.38 s main library |
| exact host MCP STDIO reconciliation | 2/2; two HTTP cases intentionally ignored in that tier | 2.33 s test |
| `npm run qa:security` | 104 passed; one explicit live-Keychain case ignored | 11.66 s wall |
| exact protocol export | 1/1, included in the full protocol target and previously rerun as the final protocol gate | passed |
| `npm run qa:native:build` | production web build, sidecar source verification, Rust application, bundle, and post-build resource copy passed | 140 s Rust build plus preflight/copy |
| bundled OpenCode SDK graph | 1/1 | 122 s compile; 1.29 s test |
| bundled OpenCode assets | 1/1 | 12.91 s compile; 6.69 s test |
| bundled Pi graph | 1/1 | 10.39 s compile; 11.97 s test |
| MCP Streamable HTTP | 2/2 | 3.16 s test |
| OpenCode native/bundle/loopback | 7/7 | 20.87 s test |
| authenticated OpenCode streaming | 3/3 | 0.05 s test |
| packaged production runtimes | 4/4 | 78.49 s test |
| private-TLS OpenCode/Pi golden | 1/1 | 46.80 s test |

One overly broad OpenCode-stream filter was stopped after its three library tests passed and before it enumerated unrelated zero-match integration binaries. It is not counted; the replacement `--lib` command passed exactly 3/3.

The complete Rust command wall time was not separately instrumented by the resumed coordinator. Exact compile/test subphase timings above are retained rather than inventing an aggregate.

## Native restart and recovery

The original `~/.c4os` was restored before the final runs and was not used by Task 00015. Final acceptance used only mode-0700 disposable homes:

- healthy and saturated: `/private/tmp/c4os-task15-disposable-final-home`
- intentionally degraded: `/private/tmp/c4os-task14-final-degraded-used2`

The healthy home began with the update-coordinator diagnostic journal already at the fixed 250-record maximum. Before the repair, the next startup diagnostic failed persistence, startup stopped after Browser/Terminal, and the UI incorrectly degraded at MCP.

The repair permits the atomic replacement scope to contain the bounded union of the prior and next journals (maximum 500 IDs) while keeping inserted and retained diagnostics capped at 250. The same immediate transaction still performs state-document compare-and-swap, replacement deletion, insertion, retention pruning, count pruning, and generation publication.

Final native results:

1. The repaired bundle launched the saturated home at `tauri://localhost#/onboarding`.
2. SQLite retained exactly 250 diagnostics and update generation 263.
3. The newest generation contained all nine startup boundaries: Database, Configuration, Runtime/Provider, Extension, MCP, Browser/Terminal, Credential, Workspace, and final Startup completion.
4. Controlled `Cmd+Q` left no application or sidecar process.
5. Relaunching the same home remained healthy, retained 250 diagnostics, advanced to generation 272, and again recorded all nine boundaries.
6. A final evidence launch remained healthy.
7. The preserved degraded Database fixture exposed only Retry and Refresh, no Continue or restore action, and blocked every product route.
8. Retry minted a new correlation and remained blocked because the Database failure was intentionally unresolved.

## Redaction, logs, and cleanup

- The isolated healthy home contained no credential-shaped value; onboarding remained in explicit session-only fallback with no provider key entered.
- Diagnostics are structured, bounded, path-free projections. The saturated journal rolled without exposing an identifier, candidate digest, filesystem path, environment value, or credential.
- The private-TLS wrapper removed its certificate/key directory and terminated all captured descendants.
- Final process scans found no C4OS, OpenCode, Pi, MCP fixture, Cargo, or rustc process.
- The golden provider evidence was restored to its pre-run tracked content.
- Unified logs contained Apple/WebKit sandbox, AppIntents, and Computer Use/AppKit geometry notices, including negative-geometry notices during accessibility capture. No Rust panic, renderer exception, product recovery error, or credential value appeared.
- The unrelated generated `MarketplaceSnapshot.ts` trailing whitespace remains outside Task 00015; staged Task 00015 content has no whitespace error.

## Independent Agent Acceptance

| Owner | P0 | P1 | P2 | P3 | Result |
| --- | ---: | ---: | ---: | ---: | --- |
| Rust/native/security | 0 | 0 | 7 | 0 | passed |
| Renderer/integration | 0 | 0 | 2 | 0 | passed |
| Policy/configuration | 0 | 0 | 2 | 0 | passed |

Retained Rust/security P2s:

1. Security current-state capacity fails closed at 4,096 instead of compacting terminal rows.
2. Generic journal validation does not deserialize every record kind into its typed payload.
3. Terminal policy metadata overstates retained-shell project containment.
4. A crash after encrypted provider credential creation but before durable profile publication can leave an encrypted, authority-free orphan.
5. Validated-backup recovery is not normally reachable when Database startup itself fails.
6. A post-effect direct-action journal-completion fault can consume the retryable lease.
7. MCP cancellation can arrive after the final check but before Ready commit; the registry remains quiesced, but teardown may await disable/restart.

Retained renderer P2s:

1. A cast can mutate the QA adapter's live replay-event array despite its readonly surface; product UI paths consume copies.
2. The fixed QA identity pill can visually occlude top-right QA chrome at narrow widths, while `pointer-events: none` preserves operation.

Policy closeout retains the Terminal containment-metadata overstatement and conservative sticky policy-recovery projection as explicit limitations. The earlier MCP terminal-commit recovery finding is closed by the tested worker close/recoverable projection. The encrypted orphan credential contains no plaintext and grants no route authority.

## Evidence images

| File | Purpose | Dimensions | SHA-256 |
| --- | --- | --- | --- |
| `task-00015-route-launcher.jpg` | wide deterministic route launcher | 1100×761 | `d38d7f6b81831afb4b277b92cf720dc385a025118c1678fd0dd3f7f72faa69f8d` |
| `task-00015-route-launcher-narrow.jpg` | narrow deterministic route launcher | 626×761 | `b2b0f44682aed4e3fb21bea516939ef3e7a98c63b83618792d5a2d4a559abf8d` |
| `task-00015-native-light-capabilities.jpg` | Light native capability composition | 1101×761 | `9050d63740f99a306cc2db08f15dc65da300a97d4b05682611934c7cade212b2` |
| `task-00015-native-dark-capabilities.jpg` | Dark native capability composition | 1101×761 | `85dfea0cc5b0128a5a0642a6fa7784da72b656f65bf417d0dc406555b18001db` |
| `task-00015-keyboard-capabilities-narrow.jpg` | narrow keyboard focus evidence | 626×761 | `2d9e0ccb48bc9dd5d989c9201230281cb0383b1114904fe3b7d5d30197a2bdfd` |
| `task-00015-final-startup-recovery-degraded.jpg` | blocked degraded startup state | 1100×761 | `a8dff96e0c6914e1b1a49cbfb5571996cdcf530c86529b0265692c7d72396388` |
| `task-00015-saturated-restart-healthy.jpg` | healthy production launch from a saturated journal | 1100×761 | `199c1cf8ca5ec0afa94c0d67dbe359464fda79ef3577001468d8263c70438450` |

## Closeout boundaries

- All 50 Feature Coverage IDs close only with Tasks 00015 through 00015C verified and passed.
- Context promotion was reviewed and requires no change: QA mechanics and bounded database transaction details are implementation evidence, not new reusable product truth.
- Signing, notarization, distribution, signed update feeds, public marketplace governance, Windows/Linux claims, Codex import, active cross-runtime migration, named unvalidated SSH, detached Chat, multiple Reply targets, Browser sub-tabs, and Terminal full-screen/password entry remain external or deferred gates.
- No branch push or pull request was performed.
