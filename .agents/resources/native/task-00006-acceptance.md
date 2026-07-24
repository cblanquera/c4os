# Task 00006 Renderer Shell Acceptance Evidence

Date: 2026-07-21

Target: macOS arm64, production-composed debug `C4OS.app` plus build-gated browser QA projections

Checkpoint base: `c35b702`

## Golden path

1. One React Router 8 hash data router directly addresses all 16 accepted product routes.
2. One Redux store owns authoritative per-domain projections, local drafts, build-gated QA state, and the Settings return record.
3. `/chat` renders the stateful desktop shell with one left project/context panel, one center stage, one fixed composer, and no right panel.
4. QA/review entry visits Providers inside the same Settings shell; native `Cmd+,` enters that same route without a production-only web control.
5. Back returns to the exact accepted route while retaining composer draft, panel state, and a stable focus target.
6. Unknown routes, stale projection generations, unavailable service content, deferred browser/terminal/reply behavior, local persistence authority, and arbitrary renderer IPC all fail closed.

## Automated verification

All source-changing lanes were frozen before the full matrix. Cargo commands ran one at a time.

| Gate | Result |
| --- | --- |
| `npm run format:check` | passed in 3.80s after the final viewport-reconciliation repair |
| `npm run lint` | passed with zero warnings in 7.90s |
| `npm run typecheck` | passed in 3.80s |
| Focused Task 00006 Vitest matrix | route controller, native bootstrap, and panel reducers, 12/12 passed in 2.98s; the earlier 10-file shell matrix also passed 37/37 |
| `npm run test:unit` | 26 files, 106/106 passed in 8.67s |
| `npm run build:web` | passed in 3.26s; production bundle excludes QA fixture state |
| `npm run build:qa` | passed in 2.95s |
| Focused viewport Playwright regression | 1/1 passed in 7.70s after exercising 704px at 1280px, 340px at 760px, and 280px at 700px |
| `npm run test:e2e` | 26/26 passed in 13.79s; the earlier sandboxed attempt could not bind the loopback QA server and its permitted rerun passed unchanged |
| Curated screenshot regeneration | 2/2 focused Playwright cases passed |
| `cargo fmt --all -- --check` | passed in 0.84s |
| `cargo clippy --workspace --all-targets -- -D warnings` | passed in 61.45s |
| `cargo test --workspace --all-targets -- --test-threads=1` | all runnable tests passed in 239.11s; explicitly opt-in loopback, live-Keychain, native, and bundle tiers remained ignored |
| `cargo audit` | no vulnerabilities; 17 allowed upstream maintenance warnings |
| `npm run tauri:build` | passed in 120.26s after the final renderer change; exact debug app and sidecars bundled |
| `npm run bundle:opencode-sdk:verify` | passed in 45.73s |
| `npm run bundle:opencode-assets:verify` | passed in 10.19s |
| `npm run bundle:pi:verify` | passed in 17.11s |
| `git diff --check` | passed |

The renderer boundary test scans production TypeScript/TSX and permits generic Tauri `invoke` only in `src/platform/native-transport.ts`. It also rejects `localStorage`, `sessionStorage`, and `dangerouslySetInnerHTML` in the production renderer. Route-contract tests require exactly the 16 accepted direct paths and reject detached-window, browser-sub-tab, and full-screen-terminal routes.

## Browser QA matrix

- All 16 product routes were loaded directly and exposed one accessible route heading without document overflow.
- Settings Back retained the exact `/chat` route, a changed composer draft, collapsed panel state, and focus on the review Settings trigger.
- The project-panel resizer passed keyboard range/value behavior; the responsive panel passed docked, collapsed, overlay-open, and outside-dismiss paths.
- The project-panel range and reducer share the same computed 55% maximum. Retained wide widths reconcile immediately in rendered ARIA state and then in Redux at every viewport resize, including changes within one breakpoint; the mobile overlay can also reopen after a docked panel was collapsed before crossing the breakpoint.
- Widths 1440, 993, 992, 680, 620, and 390px retained the fixed composer, one center stage, and no right panel.
- Settings compressed to a 64px navigation column at 620px and retained independently scrolling content.
- Dark appearance, reduced motion, long content, console-error, and document-overflow checks passed.
- Contextual Chat and its disabled `Detach Chat` affordance appear only when an artifact is focused; the default `/chat` shell gives the full panel height to projects. Multiple Reply targets, browser sub-tabs, full-screen terminal programs, and terminal password entry remained visible where applicable and disabled with explicit unavailable copy.

## Native macOS matrix

The exact rebuilt app at `target/debug/bundle/macos/C4OS.app` was launched from a confirmed stopped state and inspected through native Computer Use.

| Check | Result |
| --- | --- |
| Production launch | passed at `tauri://localhost#/start`; the renderer consumed the typed native workspace snapshot and rendered `Ready — Choose a workspace to continue.` |
| Native window/menu | passed; standard close/full-screen/minimize controls and C4OS, File, Edit, View, Window, Help menus were exposed |
| Native Settings entry | passed; `Cmd+,` opened `tauri://localhost#/settings/providers` in the same document |
| Settings identity/navigation | passed; Providers plus Models, Runtimes, Configuration, Plugins, Skills, and MCP Servers were named in accepted order |
| Back restoration | passed at wide and 622px widths; exact `/start` route restored |
| Keyboard focus | passed; Tab moved from the web document to Back, then Providers in deterministic order |
| Native accessibility | passed for named window controls, menus, route headings, Settings controls, picker action, and route identity |
| Minimum-width Settings | passed at 622x761; navigation compressed to symbols, cards reflowed, and content remained vertically scrollable without horizontal clipping |

The final post-repair `Cmd+Q` closed the acceptance window and the exact app process exited. An earlier pre-repair run removed the window but left production-runtime initialization headless; that exact PID was terminated with `SIGTERM`. The final clean exit does not erase that intermittent observation: it remains recorded for the later runtime diagnostics/final native audit rather than being represented as shell behavior.

## Review artifacts

- `task-00006-chat-settings-roundtrip.png` — deterministic default `/chat` shell after Settings Back, exact draft retention, full-height project panel, fixed composer, disabled Reply target, no contextual Chat without artifact focus, and no right panel.
- `task-00006-settings-compressed.png` — 620px QA Settings composition.
- `task-00006-start.png` — exact production native launch at 1100x761.
- `task-00006-settings.png` — exact production native Settings at 1100x761.
- `task-00006-settings-narrow.png` — exact production native Settings at 622x761.
- `task-00006-start-narrow.png` — exact production native Back restoration at 622x761.

## Scope and limitations

- QA projections are compiled only when `VITE_C4OS_QA_FIXTURES=1`; production launch showed no QA workspace, conversation, or review Settings control.
- Production native bootstrap reads only the existing allowlisted native platform, runtime, and workspace snapshots and publishes the available typed domains into Redux. Sources that are unavailable remain uninitialized and therefore fail closed; no renderer fixture or local persistence authority fills them in.
- Production native launch currently stops at the ready Workspace Start route because no workspace is selected. The full `/chat` state-retention walkthrough therefore uses the production-composed shell with build-gated QA data in Playwright, while the exact native bundle proves native projection ingestion, native Settings routing, and Back at `/start`. Opening a production workspace and reaching native `/chat` belong to later service-integration tasks.
- Task 00006 establishes shell/state/route/accessibility structure. Conversation operations, artifact behavior, complete launch/workspace services, and complete Settings services remain owned by Tasks 00007 through 00013.
- No signing, notarization, distribution, push, or pull request occurred.

## Agent Acceptance

The final independent read-only review passed with P0=0 and P1=0. It confirmed typed fail-closed production ingestion, exact native appearance provenance, collapsed-to-overlay reopening, artifact-focus-only contextual Chat, and rendered/ARIA/Redux reconciliation across the retained `704→340→280` viewport path. Its only non-blocking observation is that an odd viewport can yield a fractional ARIA maximum while panel width is rounded to an integer; the rendered value remains within range.
