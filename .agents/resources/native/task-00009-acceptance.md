# Task 00009 Terminal Facility Acceptance Evidence

Date: 2026-07-22

Target: macOS arm64, production-composed debug `C4OS.app` plus build-gated browser QA projections

Verified Task 00008 base: `1a51ac5`

Mid-epoch restart checkpoint: `3877e3d`

Disposable acceptance roots:

- Complete approval, cwd, Stop, stdin, resize, and prompt-reuse matrix: `/private/tmp/c4os-task9-final.yJ97Kn`
- Final Stop, same-generation shell reuse, and focused restart matrix: `/private/tmp/c4os-task9-final2.B2OvHR`
- Visible inline Reply, restart Recovery, and post-Recovery reuse: `/private/tmp/c4os-task9-inline.MiWJPP`
- First independently reconciled partial-output and Chat-inactivation completion: `/private/tmp/c4os-task9-driver.0hyMFE`
- Exact-final-bundle partial-output and Chat-inactivation completion: `/private/tmp/c4os-task9-finalpass.XiVzRL`

## Golden path

1. One active Chat owns one Rust-supervised portable PTY rooted in its active Project. Rust owns process identity, the terminal-session identity, process generation, command sequence, cwd, dimensions, byte cursors, bounded output, and recovery authority.
2. A direct Terminal Run creates one durable immutable command artifact and one exact Action Gateway candidate. No command bytes reach the PTY before the explicit approval is durably consumed.
3. The approved command runs in the retained shell, emits private start/exit sentinels, streams sequenced bounded bytes, records exact process/environment provenance, and completes with an immutable exit result.
4. The same typed provider renders through the shared compact transcript artifact and focused center shell. One real Chat DOM moves to the contextual panel, the contextual composer remains Chat-owned, and Terminal Reply creates an immutable typed reference rather than mutating the source artifact.
5. Later commands reuse the same shell and cwd. Resize, raw stdin, output acknowledgement, Stop/SIGINT, Chat inactivation, restart recovery, and replacement generations extend the same production composition without granting the renderer PTY or process authority.

## Automated verification

All implementation writers were frozen before the full matrix. Cargo-producing commands ran one at a time.

| Gate | Result |
| --- | --- |
| Focused Rust Terminal projection | 16/16 passed for incremental split-trigger redaction, transactional persistence rollback, invalid-UTF-8 semantic bounds, pristine acknowledgement, durable interrupt and cleanup markers, queued/startup recovery, duplicate-approval suppression, generation-scoped drains, and durable provider-session enumeration |
| Focused real PTY/process-group integration | 11/11 passed for partial no-newline output before completion, shell reuse/cwd, stdin, resize, bounded output, Stop/130, stubborn-shell replacement, restart reconciliation, session capacity, and scoped shutdown |
| Focused renderer Terminal | 2 files, 13/13 passed; the final viewport acknowledgement slice passed 5/5 |
| Focused Terminal Playwright | 3/3 passed for approval/stream/stdin/resize/Reply, shared Workspace generation rebasing, and Stop/130/prompt reuse; an initial sandbox-only `listen EPERM` was diagnosed before the authorized local-listener rerun |
| `npm run format:check` | passed |
| `npm run lint` | passed with zero warnings |
| `npm run typecheck` | passed |
| `npm run test:unit` | 47 files, 233/233 passed; one asynchronous menu-focus assertion initially failed only in the parallel full suite, passed in isolation and with file parallelism disabled, then passed under the exact default full command after matching the existing asynchronous focus-restoration contract |
| `npm run build:web` | passed |
| `npm run build:qa` | passed |
| `npm run test:e2e -- --workers=1` | 41/41 passed, including all three production-composed Terminal paths |
| `cargo fmt --all -- --check` | passed |
| `cargo clippy --workspace --all-targets -- -D warnings` | passed |
| `cargo test --workspace --all-targets -- --test-threads=1` | passed on the final source; the library target reported 137 passed, 0 failed, and 3 explicitly ignored out of 140, and every runnable integration, binary, and example target passed |
| `npm run protocol:check` | passed; exported TypeScript bindings exactly match Rust |
| Final non-QA `npm run tauri:build` | passed after all renderer and native repairs; exact debug app and sidecars bundled |
| `npm run bundle:opencode-sdk:verify` | passed against the final app bundle |
| `npm run bundle:opencode-assets:verify` | passed against the final app bundle |
| `npm run bundle:pi:verify` | passed against the final app bundle |
| `npm audit --audit-level=high` | passed with 0 vulnerabilities; the first sandbox attempt failed only because registry DNS was unavailable and the authorized network rerun completed |
| `cargo audit` | passed with no vulnerabilities and 17 allowed upstream unmaintained/unsound warnings already present in the locked graph |
| `git diff --check` | passed |

Independent read-only race reviews initially found a duplicate post-approval prompt window and two restart/replacement-generation hazards. The production repairs now suppress requeue when the queued artifact is already bound to the exact live command, reconstruct only a strictly matching completed predecessor for an approval-waiting replacement after restart, and filter event replay by exact process generation. Later independent review found inactive-Chat reconciliation, partial streaming, invalid-UTF-8 bounds, transactional redactor state, cleanup acknowledgement/retry, root-independent startup recovery, and finite trigger-lookahead defects. The final production composition now actively reconciles every retained Terminal session on a bounded background driver, persists before acknowledgement, rolls redactor state back on failed persistence, retries cleanup independently, recovers unsupervised Running artifacts before Project/root lookup, and enumerates durable provider-owned session IDs rather than relying on the bounded active-Chat UI snapshot. Focused regressions and the complete serialized Rust matrix passed after those repairs.

Native Stop exposed two further correctness edges. The renderer had acknowledged the pristine empty output before Rust could publish a cursor, and the shell wrapper's default SIGINT behavior could discard the wrapper line and force a replacement shell after an ordinary Stop. Rust now permits only the exact pristine empty acknowledgement as a no-op, the renderer waits for native cursor authority, and the wrapper installs a scoped SIGINT trap around command evaluation. Stop therefore records one durable `^C\r\n`, returns exit 130, and leaves the same shell generation reusable.

## Native macOS matrix

The exact production-composed debug bundle launched through macOS LaunchServices with the debug-only validated `--c4os-acceptance-home` switch. Release builds reject this switch. Every acceptance home was mode `0700` and seeded only through production Workspace, database, Project, Chat, Session, and artifact services.

| Check | Result |
| --- | --- |
| Production launch/resume | passed at `tauri://localhost#/chat`; no QA fixture projection was present |
| Authorization before bytes | passed; the first Terminal artifact remained Queued with empty output and no `authorized-only` effect until explicit Allow, then the exact action advanced through consumed authorization and succeeded |
| Persistent cwd | `cd nested && pwd` completed in the nested Project directory; later `pwd` commands retained that exact cwd |
| Streaming and completion | passed with safe no-newline output visible while Running, sequenced durable output, immutable history, Completed state, exit 0, and no duplicate approval |
| Resize | passed through a distinct low-risk, generation-bound Action Gateway approval without duplicating the Run prompt |
| Raw stdin | `read answer; printf 'input:%s\n' $answer` accepted `native-input` only while Running and completed with `input:native-input` |
| Stop and exit normalization | `sleep 30` accepted one explicit Stop approval, rendered one `^C`, and completed `Interrupted · exit 130` |
| Same-shell reuse after Stop | the next `pwd` remained command sequence 3/process generation 1, completed in the retained nested cwd, and did not report shell replacement |
| Prompt continuation | passed; pristine output acknowledgement no longer caused a generic snapshot warning before the next focused prompt |
| Focus composition | passed; one Terminal owned center stage, the same Chat DOM moved to contextual Chat, and contextual Chat composition remained available |
| Reply | passed; Reply produced an immutable `Reply to terminal` composer strip quoting `$ pwd`; the source command artifact did not change |
| Restart recovery | quitting during `sleep 30` discarded persisted PID authority, retained the command artifact, and relaunched it as visible Recovery with `The previous Terminal process was not trusted after restart.` |
| Recovery replacement | the next approved `pwd` used command sequence 3/process generation 2 and completed successfully from the retained under-root cwd |
| Chat inactivation continuity | in `/private/tmp/c4os-task9-finalpass.XiVzRL`, `printf tokenized2; sleep 60; printf done2; touch final-inactivation-survived-2` entered Running at `1784656022544`, persisted `tokenized2` at `1784656022545`, and the Chat was inactivated at `1784656030797`; `done2` persisted at `1784656082559`, completion exit 0 persisted at `1784656082590`, and the zero-byte marker existed at the exact Project path |
| Background-driver independence | passed; final output arrived 51.762 s after Chat inactivation and completion was durable 31 ms later without any active renderer projection for that Chat |
| Native semantics | passed; standard window controls and C4OS, File, Edit, View, Window, and Help menus remained exposed |

The direct Mach-O path aborted during AppKit application registration in this host environment. The exact same rebuilt bundle launched successfully through macOS LaunchServices, which is the normal `.app` launch path used for the acceptance matrix. This is recorded as launch-infrastructure evidence, not a product failure.

The focused xterm record can cause the Computer Use accessibility capture request itself to time out after restart. A one-second native process sample during the first occurrence showed the app idle on its normal main event loop rather than deadlocked. The separate inline Recovery route was captured visibly, and the final exact-source diagnostic reproduced only that known capture-tool limitation. The diagnostic process was terminated by its exact PID after the UI automation channel timed out; no other process was targeted.

## Persisted audit

The final Stop/restart Workspace retains four Terminal artifacts and 15 immutable artifact events:

- `artifact-0ed0e1b5d2d049c5a56f7376449c7a48`: `cd nested && pwd`, sequence 1/generation 1, Completed exit 0 at record revision 4.
- `artifact-69c2805db5a546f0907b43fc2ecffbd1`: `sleep 30`, sequence 2/generation 1, Interrupted exit 130 at record revision 4.
- `artifact-39229a19bb80408d92030ca9e7cef03d`: `pwd`, sequence 3/generation 1, Completed exit 0 at record revision 4 in the same nested cwd.
- `artifact-0c4b462b58914130bcab51c84f85d065`: interrupted `sleep 30`, sequence 4/generation 1, Recovery at record revision 3 after process restart.

Its application security journal retains five exact Terminal actions as 25 current security records and 50 immutable security events. Every action has one Ask decision, consumed authorization, effect-finished intent, completed approval, and succeeded result. The first exact identity is action `terminal-action-9493bf0571ee4eb8af4f9a1b6bb48769` with approval `approval:885092c7962243579535a86da48d1f0c`.

The inline Recovery Workspace retains the durable generation-1 `pwd`, generation-1 Recovery for `sleep 30`, and generation-2 post-Recovery `pwd` sequence. It supplies the visible Reply and Recovery evidence but is not used for the final Chat-inactivation claim.

The final exact-bundle Workspace is `a80657e8-8a2a-439b-8137-bf33096a2a29`, Project `06cffb39-a6e4-4323-8372-672396496b25`, and inactivated Chat `6b806ff5-48a4-4c17-876b-519f21e94d02`. Its second artifact, `artifact-d6335dc005ce4523aff23471189fab5a`, has five immutable revisions: Queued; Running; Running with `tokenized2`; Running with `tokenized2done2`; and Completed exit 0. The timestamps above prove the independently active driver persisted both final output and completion more than 51 seconds after the Chat left the active projection.

## Review artifacts

- `task-00009-terminal-approval.png` — explicit native approval before Terminal bytes or effects.
- `task-00009-terminal-completed.png` — completed native command with immutable output and exit result.
- `task-00009-terminal-stop.jpeg` — native `^C` and Interrupted exit 130.
- `task-00009-terminal-stop-resume.jpeg` — next command completed in the same nested cwd without shell replacement.
- `task-00009-terminal-recovery.jpeg` — visible inline Recovery after exact process restart.
- `task-00009-terminal-partial-output.jpeg` — independently driven partial output visible while Running before command completion.
- `task-00009-terminal-inactive-chat.jpeg` — empty active-Chat projection after removing the Chat while its PTY continued.
- `task-00009-terminal-final-partial.jpeg` — exact-final-bundle `tokenized` output visible while Running.
- `task-00009-terminal-final-inactive-chat.jpeg` — exact-final-bundle inactive-Chat fallback while the second 60-second PTY command continued.
- Playwright output under `test-results/` — production-focused approval, generation rebase, and Stop screenshots generated during the passing run.

## Scope and limitations

- The native run used the real Workspace, SQLite, Action Gateway, terminal supervisor, portable PTY, process-group, artifact, conversation, and xterm composition. It did not invent a live external model/provider route.
- Persisted process IDs are evidence only and are never trusted after restart. Only a canonical cwd still beneath the exact Project root may be retained.
- Full-screen terminal programs and password entry remain explicit deferred gates from the Frozen contract. Task 00009 does not claim them.
- Browser, Plugin/Skill, MCP, complete Settings/onboarding, diagnostics/update, final security/accessibility, and integrated closeout remain owned by Tasks 00010 through 00015C.
- No signing, notarization, distribution, pull request, or deferred-scope expansion occurred. The user-requested mid-epoch restart checkpoint was pushed separately; the verified Task 00009 checkpoint remains local unless the user explicitly asks to push it.

## Agent Acceptance

Passed — P0=0, P1=0, P2=1. The independent exact-source re-audit confirmed that the active driver, persistence-before-acknowledgement ordering, retry markers, startup recovery, durable provider-session enumeration, bounded streaming/redaction, and native evidence close every Task 00009 P0/P1. The sole P2 records that Terminal policy/audit metadata currently overstates Project-root containment for an unsandboxed retained shell. Task 00015A owns that cross-cutting truthful-classification and audit correction; it does not weaken approval-before-effect or fail the Task 00009 acceptance threshold.
