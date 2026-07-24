# Task 00008 Artifact File And Folder Acceptance Evidence

Date: 2026-07-21

Target: macOS arm64, production-composed debug `C4OS.app` plus build-gated browser QA projections

Checkpoint base: `e638380`

Disposable acceptance roots:

- Complete File/Folder matrix: `/private/tmp/c4os-task8-acceptance.KAFLoF`
- Approval-restart and mode-preservation repair: `/private/tmp/c4os-task8-restart.FyHTBU`

## Golden path

1. A Rust-owned native picker issues an opaque trusted-root grant. The renderer never receives unrestricted filesystem authority.
2. The descriptor-rooted `ProjectFilesystem` reads one trusted text file into a durable, schema-versioned File artifact with resource identity, content and target versions, immutable history, and optimistic revisions.
3. One typed artifact provider renders the same File through the compact transcript shell and focused center shell while the actual Chat DOM moves into the contextual panel.
4. Edit creates a durable draft. Save prepares one exact content/hash/target request and passes it through the existing Action Gateway before an atomic compare-and-swap write.
5. Folder extends the same artifact record with bounded listings, breadcrumbs, trusted traversal, refresh, and Folder-to-File conversion. Reply captures an immutable typed context reference without cloning the live artifact.
6. Conflict, restart, degraded/unknown-version, recovery, proposal/diff, accessibility, responsive containment, and persistence tests extend the same production composition.

## Automated verification

All implementation writers were frozen before compilation and full verification. Cargo-producing commands ran one at a time.

| Gate | Result |
| --- | --- |
| Focused artifact Rust tests | passed for provider schema/persistence, hostile paths, descriptor-rooted reads, symlink and time-of-check races, atomic writes, approval allow/deny, conflicts, recovery, immutable context budgets, and runtime proposal reconciliation |
| Focused renderer tests after independent-review repairs | 5 files, 32/32 artifact, Shell controller, service, and focus tests passed; the final Shell session-isolation slice passed 9/9; the last File/Folder selection-lifecycle slice passed 12/12 |
| Focused artifact Playwright | 6/6 File read/edit/save/Reply, proposal decisions, conflict choices, Folder navigation, real-DOM focus, and responsive paths passed |
| `npm run format:check` | passed |
| `npm run lint` | passed with zero warnings |
| `npm run typecheck` | passed |
| `npm test` | 45 files, 216/216 passed after the independent-review repairs |
| `npm run build:web` | passed |
| `npm run build:qa` | passed |
| `npx playwright test` | 38/38 passed in 14.1s after the independent-review repairs; the initial sandbox-only `listen EPERM` was diagnosed as a pre-test local-listener denial and rerun with the required permission |
| `cargo fmt --all -- --check` | passed |
| `cargo clippy --workspace --all-targets -- -D warnings` | passed |
| `cargo test --workspace --all-targets -- --test-threads=1` | 112 library tests (109 passed, 3 declared opt-in tiers ignored) plus every runnable integration target passed; only explicitly declared native/bundle tiers remained ignored |
| `npm run protocol:generate` | passed; generated artifact/context bindings were refreshed |
| Non-QA `npm run tauri:build` | passed on the final exact source after all review repairs; exact debug app and sidecars bundled |
| `git diff --check` | passed before and after evidence finalization |

The first native File selection exposed a real long-lived-picker generation race. The production controller now rereads the current artifact Workspace snapshot after the picker returns while leaving the opaque grant unconsumed until the exact compare-and-swap request. A later focus walkthrough exposed the inverse shared-generation race as a visible conversation autosave warning. Artifact operations now rebase the conversation adapter before focus-driven autosave, and every artifact mutation rereads its current artifact generation before deriving a revision. Focused tests, the complete browser matrix, a rebuilt exact app, and repeated native transitions passed after both repairs.

Independent review initially reported P0=0, P1=4, and P2=3. The repaired production path now preserves existing POSIX mode during atomic replacement; isolates artifact state by active Session; captures and validates exact File selections and Folder entries for Reply/expansion; reconnects generated protocol bindings; corrects File validation classifications; and projects Folder lifecycle state. The full Rust gate, focused renderer slices, complete frontend/browser matrix, and exact bundle all passed after those changes.

The re-audit found two remaining selection-lifecycle edges: a Folder Reply could retain a no-longer-visible entry after in-place navigation, and File Reply could accept matching page-global text outside the active File body. The final renderer repair rebases retained Folder focus against every current listing and constrains File selection to a non-empty DOM Range contained by the active File body. Positive and external/stale negative regression cases passed before the complete frontend/browser matrix and exact bundle were repeated.

The first repair attempted to recover the original Action Gateway prompt after restart. Native verification rejected that approach: gateway restart correctly cancels open prompts because plaintext actions and targets are intentionally non-durable. The final design instead persists only a `pendingSaveRequestedAtMs` marker with the exact durable artifact candidate before policy evaluation. Restart revalidates the active Workspace, Project, Session, artifact revision, live target version, relative path, content digest, byte length, and current authority, then creates a fresh forced-Ask action and prompt. A new Deny ceiling still wins; a newly allowing policy cannot silently execute the interrupted write.

## Native macOS matrix

The exact rebuilt executable at `target/debug/bundle/macos/C4OS.app/Contents/MacOS/C4OS` launched with the debug-only validated acceptance-home switch. Release builds reject this switch. The acceptance home was mode `0700` and seeded only through the production Workspace, database, Project, Chat, Session, and artifact services.

| Check | Result |
| --- | --- |
| Production launch/resume | passed at `tauri://localhost#/chat`; no QA fixture projection was present |
| Trusted-root picker | passed through the real macOS File and Folder panels against `/private/tmp/c4os-task8-acceptance.KAFLoF/project` |
| File read | `docs/plan.md` rendered as read-only version 1 with trusted breadcrumbs and exact contents |
| Action Gateway save | explicit approval completed; the atomic write produced version 2 and SHA-256 `d54325fdd260ce9531da7f85d53278f87a76a1a08cc29e97797ea7fdbe324c2b` |
| Security audit | one action advanced through ask, authorization consumed, effect finished, approval completed, and result succeeded; 10 immutable security events remained for the one action |
| Focus composition | passed; one File owned center stage, the same Chat DOM moved to contextual Chat, composer mode was locked, and no shared-generation warning appeared |
| Folder root | passed; the bounded project listing exposed only `docs` and `README.md` |
| Folder traversal | passed through `docs/nested`; breadcrumbs remained project-relative and the nested listing exposed one `notes.txt` entry |
| Folder-to-File conversion | passed; opening `notes.txt` converted the durable provider state to File version 4 without widening the trusted root |
| Reply context | passed; Reply on `notes.txt` created a visible typed `Reply to file` reference and retained the contextual artifact |
| Optimistic conflict | passed; an external `plan.md` change produced `Save conflict`, exposed live version 3, and offered only `Reload current` or `Keep draft` |
| Conflict retention | passed; `Keep draft` retained `Conflict draft retained by C4OS.` while rebasing its live version to the external file |
| Restart continuity | passed; process relaunch restored File version 3, the unsaved draft, focused File, transcript artifacts, and `notes.txt` Reply reference |
| Pending approval restart | passed in the rebuilt app; the original prompt `approval:d1fe6f4c2c5a4a198d1db2f96fc945e4` was cancelled during fail-closed gateway restore and the artifact marker produced fresh prompt `approval:b8c117b967a14aa4b9e164cc95a40574` for a newly bound action |
| Recovered approval completion | passed; the fresh prompt advanced from pending to completed, its action result succeeded, the marker cleared, and the File artifact advanced from revision 4/version 1 to revision 5/version 2 |
| Atomic mode preservation | passed; `docs/plan.md` was mode `0751` before Save, remained `0751` while the approval was pending, and remained `0751` after the 69-byte atomic replacement |
| Native semantics | passed; standard window controls and C4OS, File, Edit, View, Window, and Help menus remained exposed |

## Persisted audit

The closed acceptance app left one Workspace artifact state at revision 4, two current artifact records, and 12 immutable artifact events.

- `artifact-d2917db2fcfc4f80814316ff41d2d15d` is `docs/plan.md`, current provider `file`, record revision 8, live resource version 3, and retains two historical File versions plus the dirty conflict draft.
- `artifact-1ba53e17c54b41b8ad18fc43316c5e40` records Folder revisions 1 through 3 and File revision 4 for `docs/nested/notes.txt`, proving durable Folder-to-File conversion without changing artifact identity.
- The live external `plan.md` SHA-256 is `cf80ac6a0c16ef47410a2f1195022cd7a387692326113517b0224560d13ec290`; the persisted draft remains separate and therefore did not overwrite that target.
- Every artifact record and event carries a canonical-document SHA-256 and positive schema/provider/revision metadata.
- The restart-repair Workspace retained `pendingSaveRequestedAtMs=1784626331065` at File revision 4 while the original prompt was interrupted. After the fresh approval completed, revision 5 had a null marker and live sequence 2.
- The restart-repair app audit keeps both identities: original action `file-write-15c39fa126934e9d8e3f34561d89602b`/prompt `approval:d1fe6f4c2c5a4a198d1db2f96fc945e4` is cancelled, while fresh action `file-write-f9f61f479d0d4efebc313e2a5570e16f`/prompt `approval:b8c117b967a14aa4b9e164cc95a40574` is completed with a succeeded result.

## Review artifacts

- `task-00008-file-read.png` — native read-only File version 1 and trusted breadcrumbs.
- `task-00008-file-approval.png` — explicit Action Gateway approval before the first effect.
- `task-00008-file-focus.png` — one focused File plus contextual Chat.
- `task-00008-folder.png` — focused bounded project Folder listing.
- `task-00008-folder-nested-file.png` — nested Folder traversal converted to `notes.txt` File.
- `task-00008-file-reply.png` — typed File Reply reference in the composer.
- `task-00008-file-conflict.png` — native optimistic conflict with only reload/keep choices.
- `task-00008-restart.png` — exact process restart with version 3 and unsaved draft restored.
- `task-00008-approval-before-restart.png` — exact draft and original approval pending before process quit.
- `task-00008-approval-after-restart.png` — the same exact candidate under a fresh actionable approval after process relaunch.
- `task-00008-mode-preserved.png` — completed File version 2 after the recovered approval; shell evidence records mode `0751`.

## Scope and limitations

- The native run used real picker, filesystem, SQLite, artifact, conversation, and Action Gateway services. It did not invent a live model/provider route. File proposal creation and bounded diff reconciliation are covered by the real runtime-dispatch Rust tests and the passing production-adapter Playwright path rather than a live external provider call.
- Git-aware presentation remains read-only unless a later explicitly authorized broker operation is requested. This task performed no repository mutation through the artifact facility.
- Terminal, Browser, Plugin/Skill, MCP, complete Settings/onboarding, diagnostics/update, final security/accessibility, and integrated closeout remain owned by Tasks 00009 through 00015C.
- No signing, notarization, distribution, push, or pull request occurred.

## Agent Acceptance

Initial independent result: failed with P0=0, P1=4, and P2=3. After the scoped repairs and one edge-case re-audit, the same independent reviewer reported **passed: P0=0, P1=0, P2=0**. Task 00008 therefore satisfies the coordinator Agent Acceptance gate.
