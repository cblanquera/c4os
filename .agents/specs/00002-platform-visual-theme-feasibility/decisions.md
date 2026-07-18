# Decisions And Gaps

## Accepted Constraints

### D-001 — Preserve the system-following product direction

State: Accepted input from the usability KB

C4OS follows the host platform's visual language and current system light/dark preference. No manual C4OS-only theme override is introduced in the reconstructed r012 scope. This spec validates implementation and fallback boundaries; it does not reopen that product choice.

### D-002 — Require target-scoped evidence

State: Accepted research boundary

Documentation may establish API availability, but native appearance and webview behavior require evidence on each claimed target. A pass on one operating system cannot silently establish another. Missing target evidence blocks only that platform-specific claim or release target.

### D-003 — Preserve semantic intent when native values are unavailable

State: Accepted research boundary

Platform-derived colors, fonts, materials, radii, controls, and scrollbars are preferences, not permission to produce inconsistent or inaccessible UI. A neutral semantic-token fallback remains valid when a native value is unavailable or unreliable, provided the fallback is explicit and passes contrast/state acceptance.

### D-004 — Require an independent macOS live webview theme listener

State: Accepted from macOS P-001 and user closeout

Use a native current-theme snapshot for macOS initialization when available. Independently listen to live webview `prefers-color-scheme`; Tauri's native `WindowEvent::ThemeChanged` did not fire and cannot be the sole live source. Preserve application state across the change.

### D-005 — Accept observed startup appearance as the feasibility threshold

State: Accepted from macOS P-001 and user closeout

The proof launched in the correct visible scheme and retained state during a live change. Instrumented first-frame timing is optional later implementation QA, not a Freeze blocker. Production still resolves and advertises the scheme before revealing the shell.

### D-006 — Accept source-qualified macOS inputs and semantic fallbacks

State: Accepted from macOS P-002 and user closeout

AppKit, webview media, font/UA, and fallback-token inputs remain distinguishable. Explicit light/dark fallback text pairs must meet the tested 4.5:1 threshold; unavailable native inputs do not justify guessed platform values.

### D-007 — Use native menus and standard macOS decorations as the baseline

State: Accepted from macOS P-003 and user closeout

Use the native application menu, `Cmd+,` for Settings, and standard window decorations. Transparent, overlay, moved-traffic-light, or custom-titlebar behavior requires a separate option-specific Proof.

### D-008 — Defer Windows and Linux to a separate spec

State: Accepted by the user 2026-07-18

Windows and Linux research is preserved as input, but their builds, native execution, visual acceptance, release matrix, desktop/compositor variations, and fallbacks are not Frozen here. They remain `not run` and must be resolved in a separate spec without inheriting macOS evidence.

## Gap Closeout

| ID | Answer | State |
| --- | --- | --- |
| GAP-001 | Native snapshot for initial macOS state; independent live webview listener required. | Answered by D-004. |
| GAP-002 | Correct observed first visible scheme is accepted; instrumented timing is optional implementation QA. | Answered by D-005. |
| GAP-003 | Source-qualified macOS inputs plus explicit accessible semantic fallbacks. | Answered by D-006. |
| GAP-004 | Native application menu and standard macOS decorations are the proved baseline. | Answered by D-007. |
| GAP-005 | Linux-specific behavior is outside this Frozen scope. | Deferred by D-008. |
| GAP-006 | This spec freezes macOS 26.5.1 arm64 evidence only; the non-macOS release matrix moves to a separate spec. | Deferred by D-008. |

## Conflict Handling

If evidence makes a concrete rule in the platform reference infeasible, do not silently weaken Context. Record the conflict here, propose the narrowest fallback, and obtain user acceptance before revising accepted reusable truth.
