# Task 00022 r013 Review Package

Status: ready for explicit user review; not accepted

Date: 2026-07-28

This is the integrated review package for corrective Tasks 00016 through 00022. It distinguishes the deterministic QA renderer, the real production macOS bundle, and the r013 visual reference. QA evidence is visibly labeled and is not production state. Reference captures express accepted visual intent and are not implementation evidence. The exact dimensions and SHA-256 values for every current capture are in the [Task 00022 evidence manifest](task-00022-evidence-manifest.md).

## Source And Bundle Identity

- Git base revision: `336dd5e8f53fed85f02adf692afb0e9b53cc59b0`
- The worktree is intentionally uncommitted and unstaged. The complete `src/frontend`, `src/backend/src`, `src/backend/tests`, and `tests` source snapshot hashes to `8e83bc20e1fc0062d213e0fb455bde5b96558498d8f455b4b43bbbd1d16239e7` using `find ... -type f -print | sort | xargs shasum -a 256 | shasum -a 256`, which includes the new untracked source and test files.
- Final production debug binary: `.build/app/debug/bundle/macos/C4OS.app/Contents/MacOS/c4os`
- Latest Task 00022B production debug binary SHA-256: `0461a21b1c7cfc489bca980106c453548e36e8e2514909906208695f6d69897f`
- Binary size and build time: 119,083,896 bytes; 2026-07-28 17:20:15 +0800.
- Reproducible downstream OpenCode flavor: `c4os-auth-fd-terminal-stop.2`; patch SHA-256 `512a8a82a157b5e7952c9637b6d4a7a03535418b868f8d25b9a06540c6f6a3b1`; arm64 native binary SHA-256 `99d5d922f715ea0df9605f72849c68c0404d48b43ff319cef4cf943a0ee650e8` (137,518,946 bytes).
- The local development bundle is arm64 and ad-hoc linker-signed. Signing, notarization, distribution, and signed-updater evidence remain external gates.
- The production web bundle and copied `.app` resources contain no deterministic QA authority markers. The fixture build contains the explicit `qa-fixture-only` authority and visible `Deterministic QA fixture data · not production state` label.

## Verification Gates

| Gate | Result |
| --- | --- |
| `npm run qa:renderer` | Passed on final renderer source: formatting, lint, typecheck, 80 Vitest files / 476 tests, production web build, production QA-boundary exclusion, QA build, and fixture-authority inclusion. |
| `npm run test:app` | Passed on final source: 47/47 Playwright tests across all accepted routes, onboarding/Start, Settings round trip, explicit-Ask Provider Test, Chat/search/capabilities, runtime credential approval, responsive widths 1440 through 390, File/Folder/Browser focus, Terminal approval/stdin/Stop, reduced motion, overflow, and failure/recovery states. |
| `cargo test --workspace --no-fail-fast -- --test-threads=1` | The complete pre-final workspace source passed every non-ignored target except the nested MCP STDIO fixture under the outer Codex sandbox; its library tier reported 297 passed / 4 intentionally ignored, and the exact MCP target passed 2/2 outside that sandbox. The final restart-correlation delta then passed the complete 26/26 `runtime_dispatch` target and the rebuilt live retry. |
| `cargo test --test mcp_transport_fixtures -- --test-threads=1` | Passed: 2 STDIO tests; 2 authenticated loopback HTTP tests remain explicit opt-in ignored tiers. |
| `npm run protocol:check` | Passed: exported Rust bindings match `src/frontend/generated`. |
| `npm run bundle:opencode-sdk:verify` | Passed against the final `.app` resource tree. |
| `npm run bundle:opencode-assets:verify` | Passed against the final `.app` resource tree. |
| `npm run bundle:pi:verify` | Passed against the final `.app` resource tree. |
| `python3 .agents/scripts/validate-agent-workspace.py` | Passed with two pre-existing preferred-line-count warnings and no hard errors. |
| `git diff --check` | Passed. |
| Headed browser inspection | 1440x900, 700x840, and 390x844 product/reference captures; zero console errors or warnings in the final product session. |
| Computer Use native inspection | The QA-qualified `.app` passed Workspace Start, Chat, Settings, live Light-to-Dark appearance transition, exact Settings Back-state restoration, an explicit-Ask Provider Test with no pre-approval persistence, and the production-composed runtime credential approval dialog at 1100x761. Before Task 00022A, the production `.app` launched without a QA marker, truthfully exposed the locked-Keychain recovery path in Light and Dark, then used an unlocked disposable namespace to pass normal Test/Continue and restart read-back without a C4OS approval. The final real-home pass then completed user-entered OpenRouter onboarding, secure-storage recovery/read-back, one bounded fresh Chat, a full rebuild/relaunch, and one successful retry of the previously failed durable Chat. Historical captures remain credential/persistence evidence, not evidence of the corrected onboarding geometry. System appearance remains Dark. |
| Process lifecycle | Controlled quit/relaunch replaced the prior C4OS/OpenCode process generation before restart acceptance. The final rebuilt production app intentionally remains open on the successful retried Chat for user review. Unrelated user processes were not touched. |

The first complete Rust run found one stale test that still expected the old preset-driven provider prompt. The test now installs an explicit `Ask` rule, proving the r013 direct-intent behavior removes only the preset prompt while preserving explicit policy interaction. Its focused rerun and the final complete workspace rerun passed. The MCP STDIO fixture failed only inside the outer host sandbox and passed the exact host-level rerun. The unlocked production-native walkthrough then exposed a stale pre-effect Test retry and a mediated Continue generation mismatch; both were repaired, regression-tested, rebuilt, and re-exercised through successful restart read-back. Task 00022A subsequently removed the user-reported picker/default panel, added supported-feature ranking in the Rust authority and frontend fallback, reran the complete gates, rebuilt production, and refreshed the successful-Test captures and first-run recording. Task 00022B repaired the user-reported disabled Continue path by migrating the exact legacy comment-only app configuration placeholder, retaining the successful Rust-owned transient test until durable commit, cleaning newly stored credentials on pre-commit failure, and preserving an enabled retry. Three production-wired Rust regressions directly cover same-token serialization, pre-commit token retention plus new-credential cleanup, and ignored post-commit reload failure.

The final real-provider run then closed three integration-only seams: a public non-secret bootstrap now activates the private OpenCode credential operation channel; the pinned downstream OpenCode build stops after a terminal provider response without a pending tool call; and existing-chat dispatch recreates process-local native session correlation after restart. The fresh Chat and post-restart Retry each produced exactly one OpenCode user message, one assistant message, one `step-start`, and one `step-finish: stop`. The final Retry completed as C4OS attempt `attempt-7f3a34fc-af22-47b6-94c6-34fc1f1f110c` in runtime generation 2 with 32 durable events. The refreshed production bundle and all three packaged validators pass. The final independent re-audit of the fresh-registry regression and its peer-specific correlation/route wording reports P0=0/P1=0/P2=0.

## Journey And Difference Matrix

| Surface | Production implementation result | r013 relationship |
| --- | --- | --- |
| Onboarding | Compact provider form; transient Test; automatic strongest-supported viable model selection with no picker or default-confirmation step; Continue persists once; truthful secure-storage recovery. | r013 provider/status/action hierarchy retained. Native semantic controls, credential recovery, service results, and the QA label are intentional additions. |
| Workspace Start | Three concise entry actions and bounded recent-workspace rows with Open or Locate. | Near-parity shape and density. Raw filesystem paths remain intentionally absent. |
| Settings | 224px navigation, 920px content cap, one route-owned title, compact route ownership, Providers/Models/Runtimes/Configuration and Plugins/Skills/MCP grouping. | r013 hierarchy retained; real service availability, approvals, failure/recovery, and policy content replace illustrative data. |
| Workspace and Chat | 228px resizable Project panel, centered title, compact search, fixed raised 760px Composer, C4OS/model identity, all four approval presets, model/provider navigation, capability conflict actions, Reply, and four modes. | r013 composition retained while durable Workspace, capability, policy, branch, and service authority remain real. |
| Artifacts | One shared File/Folder/Browser/Terminal shell; direct focus/restore; one moved Chat DOM; contextual Chat height defaults to 40% and is keyboard/pointer resizable to 60%; responsive overlay at the accepted breakpoint. | Shared r013 frame, icon, action, and contextual geometry retained. Provider-specific native state, approvals, conflicts, Browser isolation, and Terminal process state remain authoritative. |
| Responsive and platform | Wide, minimum-center, overlay, 390px, reduced-motion, long-content, keyboard, focus, renderer Light/Dark, and live native Light/Dark surfaces passed. Native text-size adjustment was not separately toggled; component and Playwright containment checks cover scaling behavior. | Native typography, controls, decorations, focus rings, and system appearance are adaptations rather than pixel substitutions. |

Intentional remaining differences are limited to platform-native metrics and controls, real-service state and failure/recovery, security/approval and capability truth, safe omission of raw paths/secrets, visible QA qualification only in QA builds, and already-deferred product gates such as detached Chat, Browser sub-tabs, multiple Reply targets, and full-screen/password Terminal behavior.

## Evidence Coverage

The exact [evidence manifest](task-00022-evidence-manifest.md) contains:

- pre-Task22A native production Keychain Test, Continue, restart read-back, and recovery evidence for credential/persistence behavior only, plus QA-native Workspace Start, Chat, and Settings Light/Dark pairs;
- QA-native and browser captures of the explicit-Ask Provider Test and runtime-initiated credential approval paths;
- blank, refreshed no-picker successful, failed, zero-usable-model, wide, and narrow onboarding captures plus the refreshed complete first-run recording;
- all eight accepted Settings destinations at wide and narrow widths, the shared provider dialog, and Providers Light/Dark pairs;
- Chat conflict, model/provider browsing, approval presets, information, Reply, all four composer lanes, and the mode popover;
- current File proposal/conflict/focused, Folder inline/overlay, Browser contextual/overlay, and Terminal stdin/Stop representatives;
- matched r013 comparison inputs.

Artifact evidence is deliberately layered. The corrective pass freshly captures the shared shell and representative material states; it does not claim that every provider-specific state was freshly screenshotted. The unchanged File/Folder, Terminal, and native Browser authorities retain their broad native approval/denial/error/recovery/restart evidence in `task-00008-acceptance.md`, `task-00009-acceptance.md`, and `task-00010-acceptance.md`. The current 47-test production-composed Playwright matrix re-exercises those integrations after the corrective shell changes.

The independent read-only audit initially found two P1 issues: a duplicated Providers title and an evidence package that was narrower than its material-state wording. The Providers component now defers the page title/support line to the route shell, with a regression test; the expanded manifest and layered claim above resolve the evidence finding without overstating what was freshly captured. Its follow-up found one remaining P1 evidence gap for Task 00016's explicit-Ask provider action and runtime-initiated credential action. Both are now exercised through build-gated QA adapters and captured natively and in Playwright with visible deterministic-state qualification. The native runtime capture mounts the production approval center; the browser companion independently verifies the same decision through the QA-composed panel. After that distinction repaired the sole P2 wording qualification, the pre-Task-00022B independent verdict was P0=0, P1=0, and P2=0.

## Open Acceptance Gate

The normal production-native Keychain create/read/restart path used only a disposable item and dummy credential; exact cleanup removed the item and isolated home without changing the real C4OS installation key. Because that native success screenshot predates Task 00022A and visibly contains the rejected panel, it supports only the credential/persistence path. The current wide and narrow product captures and refreshed first-run recording show the verified no-picker state. Tasks 00018, 00022A, and 00022B are verified. The final real-home pass completed user-entered onboarding, secure-storage restart read-back, fresh Chat, and durable post-restart Retry. The rebuilt app remains open on the completed retry. Explicit user acceptance is now the only local-development gate; no task is marked accepted by this record.
