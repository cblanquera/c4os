# Proof Plan

State: Frozen 2026-07-18; macOS Proofs closed; Windows and Linux deferred to a separate spec

Proofs are required because documentation alone cannot establish startup rendering, native window behavior, webview styling, or cross-platform visual equivalence.

## Research-Loop Refinements

- Every Proof records OS release/build, architecture, Tauri version, webview version, and applicable desktop, compositor, display protocol, and portal backend.
- P-001 compares native, portal when applicable, and webview theme signals rather than treating any undocumented event path as universal.
- P-002 begins with `system-ui`/`ui-monospace` and neutral semantic fallbacks, then adds only source-qualified host inputs.
- P-003 tests a native-menu, standard-decoration baseline before transparent, overlay, or custom-titlebar candidates.
- P-004 remains sequenced after P-001 through P-003 and covers only the target matrix accepted by the user.

## P-001 — Platform, startup, and live-theme propagation

- **Gaps:** GAP-001, GAP-002, GAP-005.
- **Hypothesis:** A minimal Tauri shell can identify the target, initialize the correct system scheme before reveal, follow live theme changes, update semantic UI without navigation/state loss, and fall back to webview media queries.
- **Expected signal:** Timestamped native and webview state agree at launch; no wrong-scheme frame is captured; repeated OS theme changes update the visible test surface and preserve fixture state.
- **Failure signal:** Visible flash, mismatched native/webview state, missing event, restart requirement, state loss, or target-specific behavior presented as universal.
- **Scope:** Minimal window, platform/theme bridge, hidden-to-visible sequence, event logging, and deterministic fixture state.
- **Non-goals:** Full C4OS styling, branding, or production application integration.
- **Prototype path:** `proofs/platform-theme-propagation/`.
- **macOS disposition:** Native live-event expectation failed with an accepted fallback. Startup agreement, live webview propagation, fresh native snapshots, and fixture-state retention passed. Tauri's native `WindowEvent::ThemeChanged` did not fire, so the webview listener is required as an independent live authority/fallback. No wrong-theme frame was visible; the user accepted visual observation as the feasibility threshold without instrumented first-frame timing.
- **Evidence:** [`proofs/platform-theme-propagation/platform-theme-propagation-evidence-2026-07-18.md`](../../../proofs/platform-theme-propagation/platform-theme-propagation-evidence-2026-07-18.md)

## P-002 — System inputs and semantic fallbacks

- **Gaps:** GAP-003, GAP-005.
- **Hypothesis:** C4OS can normalize a bounded set of reliable host appearance/accessibility inputs and produce accessible light/dark semantic tokens when other values are absent.
- **Expected signal:** Each tested target reports source-qualified color scheme, accent, contrast, reduced motion, font availability, CSS system colors, control/scrollbar behavior, and explicit fallback use; all token states meet the chosen contrast checks.
- **Failure signal:** Unqualified guessed values, inaccessible fallback, assumed Linux font/toolkit state, or silent divergence between native and webview inputs.
- **Scope:** Diagnostic surface, computed styles, native/portal queries, font checks, and token output.
- **Non-goals:** Pixel-identical platform cloning or final brand palette.
- **Prototype path:** `proofs/platform-theme-inputs/`.
- **macOS disposition:** Proved and accepted. AppKit, webview media, font/UA, and semantic fallback sources remained distinguishable; all tested light and dark fallback pairs met the 4.5:1 threshold.
- **Evidence:** [`proofs/platform-theme-inputs/platform-theme-inputs-evidence-2026-07-18.md`](../../../proofs/platform-theme-inputs/platform-theme-inputs-evidence-2026-07-18.md)

## P-003 — Native menu and window-chrome behavior

- **Gaps:** GAP-004, GAP-005.
- **Hypothesis:** Native menus and standard or narrowly customized decorations can provide platform-correct Settings entry, shortcuts, dragging, focus, resize, window menu, traffic-light/caption safety, and light/dark chrome without faking native controls in content.
- **Expected signal:** The named behaviors work through real native windows on each tested target, with platform differences and any degraded fallback recorded.
- **Failure signal:** Lost native behavior, inaccessible caption controls, unsafe drag regions, incorrect menu placement/shortcut, or unsupported customization described as native.
- **Scope:** One main window, one Settings action, standard and candidate titlebar configurations, platform screenshots, and input checks.
- **Non-goals:** Complete application menu design or detached Chat lifecycle.
- **Prototype path:** `proofs/platform-window-menu/`.
- **macOS disposition:** Proved and accepted for the standard-decoration baseline. The native application menu and `Cmd+,` opened the owned Settings route while native traffic lights and window controls remained intact. Optional custom chrome was not evaluated.
- **Evidence:** [`proofs/platform-window-menu/platform-window-menu-evidence-2026-07-18.md`](../../../proofs/platform-window-menu/platform-window-menu-evidence-2026-07-18.md)

## P-004 — Representative visual acceptance matrix

- **Gaps:** GAP-003, GAP-006.
- **Hypothesis:** The semantic contract can render representative C4OS surfaces clearly across every accepted target and preference mode without hardcoded light leaks, horizontal overflow, lost focus indication, or decorative-motion dependence.
- **Expected signal:** Reviewable screenshots and automated computed-style/overflow checks cover shell, transcript, artifacts, dialogs, popovers, editor, terminal, and Settings in light/dark plus applicable high-contrast/reduced-motion modes.
- **Failure signal:** Untested surfaces, contrast failure, state conveyed by color alone, layout shift/overflow, stale theme after switching, or a target claimed without evidence.
- **Scope:** Deterministic representative surface set after P-001 through P-003 establish the platform inputs.
- **Non-goals:** Full production UI acceptance or replacing later wireframe/product QA.
- **Prototype path:** `proofs/platform-visual-matrix/`.
- **macOS disposition:** Proved and accepted for the representative matrix. All eight fixture families rendered in Light and Dark and at the configured minimum window size; 23/23 in-app runtime checks and the tested popover/dialog keyboard behavior passed.
- **Evidence:** [`proofs/platform-visual-matrix/platform-visual-matrix-evidence-2026-07-18.md`](../../../proofs/platform-visual-matrix/platform-visual-matrix-evidence-2026-07-18.md)

## Target Evidence Rule

Each proof result is recorded separately for macOS, Windows, and every named Linux environment. Unavailable targets remain `not run` and gate only the affected target claim. Simulation may validate contract logic but cannot be labeled native platform evidence. The user deferred every Windows/Linux row to a separate spec.

| Proof | macOS 26.5.1 arm64 | Windows 11 25H2 x64 | Ubuntu 24.04 GNOME/Wayland x64 | KDE/Wayland | X11 |
| --- | --- | --- | --- | --- | --- |
| P-001 | Accepted fallback | `not run`; deferred | `not run`; deferred | `not run`; deferred | `not run`; deferred |
| P-002 | Proved | `not run`; deferred | `not run`; deferred | `not run`; deferred | `not run`; deferred |
| P-003 | Proved baseline | `not run`; deferred | `not run`; deferred | `not run`; deferred | `not run`; deferred |
| P-004 | Proved representative matrix | `not run`; deferred | `not run`; deferred | `not run`; deferred | `not run`; deferred |
