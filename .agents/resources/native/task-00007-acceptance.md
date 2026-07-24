# Task 00007 Workspace And Chat Acceptance Evidence

Date: 2026-07-21

Target: macOS arm64, production-composed debug `C4OS.app` plus build-gated browser QA projections

Checkpoint base: `ff85865`

## Golden path

1. A Rust-owned active Workspace, Project, Chat, draft, and durable `SessionRecord` are projected through one typed Conversation service into the Task 00006 shell.
2. The restored transcript retains immutable user/assistant turns, selected route provenance, normalized activity, safe reasoning summaries, usage, completion, and retry/cancellation state without exposing private reasoning payloads.
3. Project/Chat navigation, search, pending first-submit promotion, attachments, Reply, model/reasoning controls, modes, branch controls, and project lifecycle operations cross generation-checked native commands rather than renderer persistence.
4. A focusable run-activity artifact moves the actual transcript into contextual Chat, owns center stage, locks composer mode to Chat, disables detach, and restores focus to its exact Expand trigger when closed.
5. The smallest native acceptance path uses an isolated mode-0700 temporary C4OS home seeded only through production Workspace, database, and Session services. The application then runs the normal non-QA renderer and native ingestion path.

## Automated verification

All implementation writers were frozen before compilation and full verification. Cargo-producing commands ran one at a time.

| Gate | Result |
| --- | --- |
| Focused P1 repair matrix | 31/31 retry projection, attachment ledger, Reply reconciliation and submit-race, controller, and protocol-boundary tests passed |
| `npm run format:check` | passed |
| `npm run lint` | passed with zero warnings |
| `npm run typecheck` | passed |
| `npm test` | 39 files, 184/184 passed after the final renderer repair |
| Focused durable Reply Playwright | 1/1 autosave-and-reload test passed; the complete conversation file now contains 6 passing paths |
| Focused shell Playwright after compact-Chat assertion repair | 10/10 passed in 12.1s |
| `npx playwright test` | 32/32 passed in 15.4s after the final renderer repair; the initial sandboxed preview bind returned `EPERM`, and the authorized local-bind rerun passed |
| Focused conversation Rust tests | 4/4 generation/route tests and 7/7 projection tests passed |
| Focused protocol regression | identity-only request validation passed while a synthetic generation-0 response remained rejected |
| Attachment-reference Rust regression | immutable ledger 3/3 plus durable removal/re-add integration 1/1 passed |
| `cargo test --workspace` | every runnable library, integration, and doc test passed; 85 library tests included 82 passed and 3 declared loopback ignores |
| `cargo clippy --workspace --all-targets -- -D warnings` | passed in 34.38s |
| `cargo fmt --all -- --check` | passed |
| `npm run protocol:check` | passed; generated TypeScript bindings have no drift |
| Non-QA `npm run tauri:build` | passed after the final renderer repair; exact debug app and sidecars bundled |
| `git diff --check` | passed |

The full Rust run retained only explicitly declared opt-in local-loopback, disposable-Keychain, bundle, and live-native runtime tiers as ignored. The sidecar test that exceeded 60 seconds remained active and completed successfully.

## Native macOS matrix

The exact rebuilt executable at `target/debug/bundle/macos/C4OS.app/Contents/MacOS/C4OS` launched with the debug-only validated acceptance-home switch. Release builds reject this switch.

| Check | Result |
| --- | --- |
| Production launch/resume | passed at `tauri://localhost#/chat`; the service-backed Workspace, Project, Chat, and durable transcript restored with no QA fixture state |
| Native request generations | passed; two exact app launches advanced durable conversation state to generation 3 without an error notice |
| Transcript/provenance | passed; user prompt, final assistant Markdown, `model-acceptance`, `open-code`, `environment:local`, and exact Workspace name remained visible |
| Safe activity | passed; reasoning summary, generic runtime activity, usage update, and completion rendered; private reasoning payload was absent |
| Focused artifact | passed; run activity owned center while the same transcript moved to contextual Chat; Detach was disabled and composer mode was locked |
| Focus restoration | passed; closing the focused artifact returned accessibility focus to the exact Expand button |
| Model availability | passed; no live production route was invented, so model selection and Send remained disabled while historical model attribution stayed visible |
| Search | passed; a non-matching query returned an explicit empty result and `Verify` returned the one normalized Chat result |
| Settings round-trip | passed; `Cmd+,` entered Providers and Back restored exact `/chat` state and search query |
| Rename dialog | passed; the text field owned initial focus with its current name selected; Escape dismissed and restored focus to the Project actions trigger |
| Durable Reply restart | passed; the production seed persisted `replyTargetId: turn:acceptance`, both exact app launches rendered the Reply strip and draft text, and the post-autosave database record retained the same target |
| Retry history | passed in the production projection contract; failed, completed-retry, and active-retry attempts remain in authoritative snapshot order while the actual active attempt stays current |
| Attachment references | passed across the durable Rust record, generated protocol, renderer draft, and submitted transcript; survivor numbers do not change and removed numbers are not reused after restart |
| Native semantics | passed; standard window controls and C4OS, File, Edit, View, Window, and Help menus remained exposed |

## Review artifacts

- `task-00007-chat-service-backed.png` — clean non-QA service-backed Chat baseline.
- `task-00007-chat-reasoning-summary.png` — explicitly safe reasoning summary plus generic normalized activity.
- `task-00007-chat-activity-focus.png` — center-stage run activity with the actual transcript in contextual Chat.
- `task-00007-chat-information-service-backed.png` — truthful runtime/environment/Workspace/model provenance.
- `task-00007-chat-search-service-backed.png` — normalized matching Chat search result.
- `task-00007-chat-reply-restored.jpg` — exact non-QA app with the persisted Reply strip and draft restored.
- `task-00007-chat-reply-restart.jpg` — same isolated home after process relaunch, with Reply target and text still present.

## Scope and limitations

- The acceptance seed proves the normal Workspace/database/session composition and renderer projection without claiming a live provider dispatch. The complete native provider/runtime path remains independently covered by Task 00004.
- The historical session retains its bound model provenance, but the restored app has no live healthy process/capability intersection; controls therefore fail closed instead of making that historical route selectable.
- Approved attachment conversion remains unavailable because current policy/resource authority does not supply a conversion materializer. The renderer offers only supported conflict resolutions.
- Terminal, Browser, File/Folder, Plugin/Skill, MCP, complete Settings, diagnostics/update, and final integrated audits remain owned by Tasks 00008 through 00015C.
- No signing, notarization, distribution, push, or pull request occurred.

## Agent Acceptance

Passed — P0=0, P1=0, P2=1. Independent review confirmed stable retry history, durable immutable attachment references, durable Reply restoration, and compare-before-replace Reply reconciliation for delayed bootstrap, attachment, submit-success, and submit-failure responses. The non-blocking P2 is limited to the lack of separate exact-native screenshots for pending, streaming, failure, and responsive states; production code and the passing 32/32 browser matrix cover those states.
