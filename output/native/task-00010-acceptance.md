# Task 00010 Native Browser Facility Acceptance Evidence

Date: 2026-07-22

Target: macOS 26.5.2 arm64, production-composed debug `C4OS.app` plus build-gated browser QA projections

Verified Task 00009 base: `ff59999`

Disposable acceptance roots:

- Chat-scoped persistent Browser Environment, hostile-page, approval, clear, Reply, theme, and relaunch matrix: `/private/tmp/c4os-task10-final5.TGjJaL`
- `None`/ephemeral Browser Environment, clear, relaunch, and destructive Close matrix: `/private/tmp/c4os-task10-ephemeral.Ac7qaK`
- Loopback hostile-page Project roots: `/private/tmp/c4os-task10-final5-project.45YpRp` and `/private/tmp/c4os-task10-ephemeral-project.qQRc7T`

## Golden path

1. One active Chat submits one direct Browser `Open`. Rust canonicalizes the address, strips query and fragment from durable display state, hashes the exact transient navigation target, creates one versioned Browser artifact, and proposes one exact Action Gateway action before navigation.
2. After explicit authorization, Rust owns one public-API `WKWebView` child within the existing focused artifact geometry. The website receives no Tauri, Wry, C4OS, custom-scheme, script-message, or page-accessible IPC bridge.
3. The native controller publishes only bounded, sanitized URL, title, loading, history, notice, and recovery state. Rust owns controller generation, event sequence, profile generation, native request identity, history verification, policy continuation, teardown, and persistence.
4. The shared artifact shell renders compact and focused Browser states, keeps one real contextual Chat composition, and creates immutable bounded Reply context from a WebKit isolated content world without capturing form values, storage, credentials, query strings, or fragments.
5. App-wide, Workspace/Project, Chat, and `None` Browser Environment resolution, Back/Forward/Refresh, popup/download/request mediation, media denial, clearing, process relaunch, recovery, and Close extend this same composition without granting the renderer or page authority.

## Dependency and Proof lock

Task 00010 first revalidated the Frozen native-WebKit boundary against the production dependency graph:

- Tauri `2.11.5`
- tauri-build `2.6.3`
- objc2-web-kit `0.3.2`
- public WebKit APIs only

The isolated native Proof ran twice consecutively and passed all eight boundary checks both times. Its deterministic Node verifier passed 2/2 after the result/version contract was aligned. The exact rerun record is `proofs/macos-wkwebview-production-boundary/macos-wkwebview-production-boundary-evidence-2026-07-22.md`; it remains feasibility evidence and is not used as a substitute for the production matrix below.

## Automated verification

All implementation writers were frozen before the full matrix. Cargo-producing commands ran one at a time.

| Gate | Result | Measured time |
| --- | --- | ---: |
| Focused Rust Browser artifact/controller projection | 24/24 passed, including canonical identity, state transitions, bounded history, stale generation/event rejection, Reply capture bounds, secret exclusion, native request sanitization, redirects, permission policy, and teardown | 43.81 s |
| Browser profile integration | 9/9 passed for App-wide, Workspace/Project, Chat, and `None`; exact active App < Workspace < Project < Chat precedence and persistent-scope selection passed | 2.56 s final suite; 24.15 s exact regression |
| Focused renderer Browser plus viewport authority | 12/12 passed for compact/focused composition, controls, native geometry, mutation remeasurement, clear rebasing, notices, Reply, recovery, and teardown | passed |
| `npm run format:check` | passed | 5.01 s |
| `npm run lint` | passed with zero warnings | 10.16 s |
| `npm run test:unit` | 50 files, 249/249 passed; the existing canvas warning remained non-failing | 9.91 s |
| `npm run typecheck` | passed | 3.26 s |
| `npm run build:web` | passed; existing size warning remained non-failing | 3.81 s |
| `npm run build:qa` | passed | 3.69 s |
| `npm run test:e2e -- --workers=1` | 41/41 passed after the sandbox-only listener `EPERM` was diagnosed and the authorized local-listener rerun completed | 14.33 s |
| `cargo fmt --all -- --check` | passed after the final source repair | passed |
| `cargo clippy --workspace --all-targets -- -D warnings` | passed | 79.67 s |
| `cargo test --workspace --all-targets -- --test-threads=1` | passed on final source; library target 159 passed, 0 failed, 3 explicitly ignored, and every runnable integration/binary/example target passed | 319.98 s |
| Final non-QA `npm run tauri:build` | passed after all Browser, configuration-precedence, and denial-continuation repairs; exact debug app and sidecars bundled | 137.73 s |
| `npm run protocol:check` | passed; exported TypeScript bindings exactly match Rust | 17.17 s |
| OpenCode SDK bundle verifier | passed | 3.94 s |
| OpenCode asset bundle verifier | passed | 2.26 s |
| Pi bundle verifier | passed | 13.92 s |
| `npm audit --audit-level=high` | passed with 0 vulnerabilities; the first sandbox attempt failed only because registry DNS was unavailable | 0.79 s authorized rerun |
| `cargo audit` | passed with no vulnerabilities and 17 allowed upstream unmaintained/unsound warnings already present in the locked graph; the first sandbox attempt could not write the Cargo advisory lock | 2.32 s authorized rerun |

The clean JavaScript matrix completed before the final two Rust-only repairs. Those repairs changed configuration composition and denial-side native-request disposal only, so the unchanged renderer matrix was not blindly rerun. The complete serialized Rust, clippy, bundle, protocol, bundle-verifier, advisory, native, and secret-absence gates ran after the final code.

The measured commands listed above consumed 11 minutes 38 seconds in aggregate when the separately measured exact precedence regression is included. Iterative implementation, compile reuse, native setup, manual walkthroughs, screenshots, and independent review were not consistently machine-timed and are not folded into that number.

## Native macOS matrix

The exact final debug bundle launched through macOS LaunchServices with the debug-only validated `--c4os-acceptance-home` switch. Release builds reject that switch. Both acceptance homes were mode `0700` and were seeded only through production Workspace, database, Project, Chat, configuration, artifact, and Action Gateway services.

| Check | Result |
| --- | --- |
| Production launch and direct Open | passed at `tauri://localhost#/chat`; one explicit native approval preceded the first network navigation, then the ordinary loopback page rendered in a real child `WKWebView` |
| Native ownership and isolation | passed; native content visually occupied the focused artifact viewport, moved/resized with the Tauri content region, survived compact/wide remeasurement, and exposed no page-accessible app IPC |
| Geometry and z-order | passed in 1100x761 wide and 792x761 compact captures; the controller applies the macOS titlebar content inset and renderer-authoritative viewport revisions without covering the shell controls or contextual Chat |
| Address, Back, Forward, Refresh | passed through exact generation-bound controller requests; the durable record retained two verified history entries and exact navigation hashes |
| Provisional and redirect navigation | passed through one-shot native request identities; redirect and policy continuations could not reuse a stale target or generation |
| Explicit navigation denial | passed; one Deny produced the scoped `Navigation blocked` notice, discarded the native one-shot request, wrote a durable denied result, and did not immediately re-propose the same request |
| Popup, download, binary, and method guardrails | passed; popups and downloads never became peer webviews or ambient files, direct download rendered `Download blocked`, non-GET and binary responses were rejected, and ordinary GET navigation remained usable |
| Media policy | passed; the effective policy denied the loopback camera request before a platform grant, and the page visibly reported `PASS camera request denied/unavailable`; no OS camera approval was required or claimed |
| Persistent profile isolation | passed for Chat scope; profile `7d8c28c2-1e77-49d7-9f56-fc76314c35ce` remained bound only to Workspace `989979c5-ee2a-4beb-8d79-e4b2d9db7668` and Chat `80ab0091-b1cb-4eb2-976c-51d6492e8d1e` |
| Persistent clear | passed; clear advanced the profile data generation to 3, retained the stable profile identity/scope, removed the hostile storage marker, and returned the artifact through visible Recovery to Ready |
| Relaunch recovery | passed; a process restart never trusted the previous native controller, published visible Recovery, recreated a new controller generation, and restored the durable sanitized address/history against the same profile generation |
| `None` environment | passed; the registry remained `generation = 1` with `profiles = []`, clear advanced only the artifact-local environment generation, the storage marker remained absent, and relaunch created no persistent profile |
| Destructive Close | passed in the `None` environment; the native surface was removed, center focus cleared, the normal composer returned, and `artifact_workspace_state` revision 4 retained `focusedArtifactId: null` |
| Reply | passed; Reply created an immutable Browser reference with bounded visible/extracted page context while excluding the hostile form value, storage marker, credentials, query, fragment, and download body/name sentinels |
| Theme and responsive state | passed in system Dark and Light appearances, with visible Ready and Recovery states in Light and wide/compact focused layouts in Dark; the system appearance was restored to Dark afterward |
| Keyboard, focus, accessibility | passed for shell controls, approval answers, Browser toolbar, contextual Chat, Reply, and Close. The native Browser container has an accessible name; native webpage traversal remains WebKit-owned rather than duplicated into the Tauri DOM |
| Console and overflow | passed; Browser content scroll remained native and contained, the app document did not horizontally overflow, no DevTools/page console bridge was exposed, and hostile page failures remained scoped notices rather than global shell authority |

## Persisted audit

The Chat-scoped Workspace retained one Browser artifact:

- Workspace `989979c5-ee2a-4beb-8d79-e4b2d9db7668`
- Project `5e4ec131-dcbe-4f68-8729-733b98523ee2`
- Chat `80ab0091-b1cb-4eb2-976c-51d6492e8d1e`
- Artifact `artifact-d78d8b5bd1f34a94a96ed53698a3e9c1`
- Record revision 51, lifecycle Ready, environment generation 3, controller generation 13
- Sanitized history: `http://127.0.0.1:43110/` and `http://127.0.0.1:43110/page-b`

Its profile registry is schema 1/generation 6 with the one exact Chat-scoped profile above at data generation 3 and lifecycle Ready. The application security journal retains 111 current records and 214 immutable events from the full native sequence. The final clean Deny is action `browser-action-bb2bbabc4efe41e0a91cd46256654d0e`, whose approval and result are both durably Denied. The subsequent exact navigation action `browser-action-eb8d05329293450cb06e58f05eeac747` retained an Ask decision, consumed authorization, effect-finished intent, completed approval, and Succeeded result.

The `None` Workspace retained one Browser artifact at record revision 11/environment generation 3 with visible `profileCleared` Recovery after the final clear. Its persistent registry is schema 1/generation 1 with no profiles, and its workspace focus record is revision 4 with no focused artifact after Close.

The final secret scan covered both acceptance homes across every SQLite database, WAL, and TOML document. No match existed for the hostile project/storage/form/redirect/download/credential sentinels, including `must-not-project`, `private-fragment`, `FORM-VALUE-MUST-NOT-CAPTURE`, `redirect-secret`, `redirect-private`, `DOWNLOAD-BODY`, `private-download-name`, `credential=`, or `final-secret`.

## Review artifacts

- `task-00010-browser-open-approval.png` — explicit approval before the first native navigation.
- `task-00010-browser-ready.png` — final wide Dark Browser with real native page content.
- `task-00010-browser-compact.png` — compact-width native geometry and contained shell.
- `task-00010-browser-navigation-approval.png` — one scoped native navigation approval.
- `task-00010-browser-navigation-denied.png` — clean explicit Deny result.
- `task-00010-browser-blocked-actions.png` — clean scoped Navigation and Download blocked notices without a global attention banner.
- `task-00010-browser-camera-denied.png` and `task-00010-browser-camera-denial-result.png` — camera request control and visible denied/unavailable result.
- `task-00010-browser-recovery-available.png` — visible process-relaunch Recovery.
- `task-00010-browser-light-recovery.png` and `task-00010-browser-light-ready.png` — system Light Recovery and Ready.
- `task-00010-browser-reply.png` — immutable Browser Reply composition.
- `task-00010-browser-ephemeral-closed.png` — destructive Close of the `None` Browser surface and restored composer.

## Scope and limitations

- The native matrix used the real Workspace, SQLite, configuration, Action Gateway, artifact, Browser profile registry, public WebKit controller, isolated-world Reply capture, renderer shell, and native geometry composition. It used a bounded loopback hostile-page fixture rather than an external website so exact request, storage, method, redirect, popup, download, media, and secret assertions remained deterministic.
- The acceptance target is the exact debug `.app`. No signing, notarization, distribution, pull request, or deferred-scope expansion occurred.
- The page itself is not projected into the Tauri accessibility tree. The Browser container and app controls are semantic, while webpage accessibility/navigation remains owned by native WebKit.
- The acceptance Workspace intentionally has no configured model route. Reply context is immutable, durable, and visibly composed, but Send remains correctly unavailable until a model exists.
- Task 00013 still owns the complete Settings UI for editing the accepted Browser Environment presets. Task 00010 proves the production configuration authority, App < Workspace < Project < Chat precedence, exact preset-to-profile selection, and native lifecycle that Settings will call.
- The 17 allowed `cargo audit` warnings are inherited transitive GTK3, `unic`, and proc-macro maintenance/unsoundness notices, not reported vulnerabilities. Task 00015A must repeat the final integrated dependency and security classification.
- Three superseded diagnostic captures (`task-00010-browser-cleared.png`, `task-00010-browser-relaunch-recovery.png`, and `task-00010-browser-binary-response-blocked.png`) are intentionally excluded because their visual state is ambiguous or includes a pre-repair stale banner. They are not evidence for any passing claim.
- Plugin/Skill, MCP, complete onboarding/Settings, update/diagnostics, final security/accessibility, integrated QA, and closeout remain owned by Tasks 00011 through 00015C.

## Agent Acceptance

Passed — P0=0, P1=0, P2=0. The final independent exact-source and evidence re-audit confirmed that the production App < Workspace < Project < Chat resolver and exact `app_wide → workspace_project → none → chat` integration test close the prior configuration-precedence P1. Direct inspection of all 13 accepted captures, both persisted acceptance homes, profile and focus records, security journals, final command/timing matrix, secret scan, and limitations closes the prior evidence-completeness P1. No residual Task 00010 P2 was recorded.
