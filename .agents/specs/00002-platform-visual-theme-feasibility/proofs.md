# Proof Plan

State: Proposed; no prototype folders created

Proofs are required because documentation alone cannot establish startup rendering, native window behavior, webview styling, or cross-platform visual equivalence.

## P-001 — Platform, startup, and live-theme propagation

- **Gaps:** GAP-001, GAP-002, GAP-005.
- **Hypothesis:** A minimal Tauri shell can identify the target, initialize the correct system scheme before reveal, follow live theme changes, update semantic UI without navigation/state loss, and fall back to webview media queries.
- **Expected signal:** Timestamped native and webview state agree at launch; no wrong-scheme frame is captured; repeated OS theme changes update the visible test surface and preserve fixture state.
- **Failure signal:** Visible flash, mismatched native/webview state, missing event, restart requirement, state loss, or target-specific behavior presented as universal.
- **Scope:** Minimal window, platform/theme bridge, hidden-to-visible sequence, event logging, and deterministic fixture state.
- **Non-goals:** Full C4OS styling, branding, or production application integration.
- **Prototype path:** `proofs/platform-theme-propagation/` after Proof approval.

## P-002 — System inputs and semantic fallbacks

- **Gaps:** GAP-003, GAP-005.
- **Hypothesis:** C4OS can normalize a bounded set of reliable host appearance/accessibility inputs and produce accessible light/dark semantic tokens when other values are absent.
- **Expected signal:** Each tested target reports source-qualified color scheme, accent, contrast, reduced motion, font availability, CSS system colors, control/scrollbar behavior, and explicit fallback use; all token states meet the chosen contrast checks.
- **Failure signal:** Unqualified guessed values, inaccessible fallback, assumed Linux font/toolkit state, or silent divergence between native and webview inputs.
- **Scope:** Diagnostic surface, computed styles, native/portal queries, font checks, and token output.
- **Non-goals:** Pixel-identical platform cloning or final brand palette.
- **Prototype path:** `proofs/platform-theme-inputs/` after Proof approval.

## P-003 — Native menu and window-chrome behavior

- **Gaps:** GAP-004, GAP-005.
- **Hypothesis:** Native menus and standard or narrowly customized decorations can provide platform-correct Settings entry, shortcuts, dragging, focus, resize, window menu, traffic-light/caption safety, and light/dark chrome without faking native controls in content.
- **Expected signal:** The named behaviors work through real native windows on each tested target, with platform differences and any degraded fallback recorded.
- **Failure signal:** Lost native behavior, inaccessible caption controls, unsafe drag regions, incorrect menu placement/shortcut, or unsupported customization described as native.
- **Scope:** One main window, one Settings action, standard and candidate titlebar configurations, platform screenshots, and input checks.
- **Non-goals:** Complete application menu design or detached Chat lifecycle.
- **Prototype path:** `proofs/platform-window-menu/` after Proof approval.

## P-004 — Representative visual acceptance matrix

- **Gaps:** GAP-003, GAP-006.
- **Hypothesis:** The semantic contract can render representative C4OS surfaces clearly across every accepted target and preference mode without hardcoded light leaks, horizontal overflow, lost focus indication, or decorative-motion dependence.
- **Expected signal:** Reviewable screenshots and automated computed-style/overflow checks cover shell, transcript, artifacts, dialogs, popovers, editor, terminal, and Settings in light/dark plus applicable high-contrast/reduced-motion modes.
- **Failure signal:** Untested surfaces, contrast failure, state conveyed by color alone, layout shift/overflow, stale theme after switching, or a target claimed without evidence.
- **Scope:** Deterministic representative surface set after P-001 through P-003 establish the platform inputs.
- **Non-goals:** Full production UI acceptance or replacing later wireframe/product QA.
- **Prototype path:** `proofs/platform-visual-matrix/` after Proof approval.

## Target Evidence Rule

Each proof result is recorded separately for macOS, Windows, and every named Linux environment. Unavailable targets remain `not run` and gate only the affected target claim. Simulation may validate contract logic but cannot be labeled native platform evidence.
