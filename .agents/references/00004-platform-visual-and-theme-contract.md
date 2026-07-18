# Platform Visual And Theme Contract

## Intent

C4OS should feel native to the operating system that hosts it: macOS on macOS, Windows on Windows, and the active desktop environment on Linux. The information architecture and behavior remain cross-platform, but typography, materials, controls, focus, menus, window chrome, and motion adapt by platform.

Do not ship a manual C4OS-only light/dark setting in the reconstructed r012 scope. Follow the system preference live.

## Theme Resolution

1. Detect the host platform in the native shell and expose `data-platform="macos|windows|linux"` on the application root.
2. Read the host color-scheme preference at launch.
3. Expose `data-color-scheme="light|dark"` and set CSS `color-scheme` accordingly.
4. Listen for live system theme changes; update without restart, route change, or loss of application state.
5. On macOS, use a native current-theme snapshot for initial resolution when available, but independently listen to webview `prefers-color-scheme` for live changes. Do not depend exclusively on Tauri `WindowEvent::ThemeChanged`; it did not fire in the accepted macOS Proof.
6. Do not flash the wrong theme during startup: advertise `light dark`, resolve platform and scheme, and initialize semantic root state before revealing the shell. The accepted macOS feasibility threshold is visual observation of the correct first visible scheme; instrumented first-frame timing remains an implementation QA option, not a blocker for this contract.
7. Persist no theme override unless a later accepted requirement introduces one.

## Evidence-Qualified Platform Scope

- Native feasibility is accepted for macOS 26.5.1 on arm64 with Tauri 2.11.5 and host WKWebView `AppleWebKit/605.1.15`.
- Theme propagation passed through live webview media-query delivery with state retention; native live-event delivery failed and has the accepted fallback above.
- AppKit and webview inputs remain source-qualified. Explicit light/dark semantic fallbacks must meet at least 4.5:1 for normal text pairs tested by the theme contract.
- Native application menus and standard window decorations are the accepted macOS baseline. `Cmd+,` routes to Settings. Transparent, overlay, moved-traffic-light, and custom-titlebar designs require a separate target-specific Proof.
- The representative shell, transcript, artifact, editor, terminal, Settings, popover, and dialog matrix passed macOS Light, Dark, and minimum-window review. This is feasibility evidence, not production UI acceptance.
- Windows and Linux native behavior is not established by the macOS result. It is deferred to a separate spec and must remain unclaimed until exact-target Proofs run.

## Semantic Tokens

Implement components against semantic tokens, not revision-era grayscale names:

- `surface-window`, `surface-sidebar`, `surface-raised`, `surface-sunken`, `surface-overlay`
- `text-primary`, `text-secondary`, `text-tertiary`, `text-disabled`, `text-link`
- `border-separator`, `border-control`, `border-strong`
- `accent`, `accent-hover`, `accent-pressed`, `accent-text`
- `focus-ring`, `selection`, `danger`, `warning`, `success`
- `shadow-window`, `shadow-menu`, `shadow-dialog`

Every token must have light and dark values. Prefer system colors or native-derived values; hardcoded values are fallbacks. Dark mode is not a simple inversion: raised surfaces become slightly lighter than the base, separators soften, shadows reduce, and semantic colors retain contrast.

## Platform Adaptation Matrix

| Concern | macOS | Windows | Linux |
|---|---|---|---|
| UI font | system San Francisco stack | Segoe UI Variable/Segoe UI | active desktop system font, fallback Noto Sans |
| Mono font | SF Mono/Menlo | Cascadia Mono/Consolas | active mono font, fallback Noto Sans Mono |
| Window chrome | native titlebar/traffic lights; toolbar respects safe inset | native caption buttons and draggable title region | native compositor decorations where available |
| Menus | native application menu; Settings uses standard shortcut | native app menu/command surface | desktop-native app menu or compact fallback |
| Controls | macOS radii, compact segmented controls and switches | Fluent-style controls, visible hover/pressed layers | toolkit-neutral controls using desktop colors |
| Focus | macOS accent ring | Windows focus rectangle/accent ring | desktop/toolkit focus ring |
| Scrollbars | overlay where the OS uses overlay scrollbars | system-width transient scrollbar | desktop-configured scrollbar where available |
| Keyboard labels | symbols such as `⌘`, `⌥`, `⇧` | names such as `Ctrl`, `Alt`, `Shift` | names such as `Ctrl`, `Alt`, `Shift` |
| Reveal action | Reveal in Finder | Show in File Explorer | Show in File Manager |

Linux cannot guarantee one visual language. Prefer portal/toolkit/system colors and the current desktop font. If unavailable, use a neutral, compact desktop fallback; do not imitate macOS or Windows.

## Structural Geometry

These r012 dimensions are reconstruction defaults, then adapt to native metrics:

- Header: 58px.
- Left project panel: 228px initial; 180px minimum.
- Left maximum: smaller of 55% of workspace width or the width preserving 420px for center content.
- Composer reading width: 760px maximum.
- Assistant/artifact reading width: `min(88%, 660px)`; narrower inline surfaces may use `min(78%, 610px)`.
- Contextual Chat pane: 40% viewport-height default, 220px practical minimum, 60% maximum.
- Inline artifact body: responsive maximum from 260px to 440px, derived from viewport height.
- Small/medium/large radii in the grayscale proof were 6/10/16px; map them to platform-native radii rather than treating the values as immutable.

Maintain the density relationships even when native control metrics shift: sidebar is compact, transcript is relaxed, settings is scan-oriented, and focused artifacts maximize work area.

## Responsive Behavior

- Desktop is the primary form factor.
- Below 992px, the left panel behaves as a resizable overlay; clicking the center dismisses an open overlay without consuming the intended center action.
- Below 860px, ensure the contextual Chat pane remains contained in the overlay.
- Below 680px, Settings navigation compresses to icons while content preserves a usable desktop-oriented minimum.
- Below 780px, Advanced Policies stacks the group rail above results and allows horizontal group scrolling.
- Below 620px, workspace start actions stack; icons align with the first title line and cards size to content with balanced padding.
- No supported width may create document-level horizontal overflow. Local code, file, terminal, table, and artifact regions scroll internally.

## Interaction Presentation

- Hover, pressed, selected, disabled, dirty, loading, success, warning, and error states must be visually distinct in both schemes.
- Hover-only action rows reserve their final layout space so appearing controls do not shift content.
- Use the platform accent for selection and focus, but keep destructive actions semantically red/danger.
- Popovers open adjacent to their trigger and remain within the window; the composer mode popover opens upward.
- Dialogs use native-feeling modal elevation, an inert backdrop, initial focus, focus containment, Escape close, and focus restoration.
- Motion is short and functional. Panel and overlay transitions may use roughly 180ms; streamed response timing is content behavior, not decorative motion.

## Icons And Branding

- Use one coherent outline icon family with approximately 1.7px stroke at the 18px base size; adapt only when the platform's native icon set is available consistently.
- Artifact type icons identify Browser, File/Folder, and Terminal in the C4OS response identity row; do not repeat them unnecessarily in the frame header.
- C4OS branding remains compact and secondary to the current task. Native window chrome must not be faked inside product content.

## Theme Acceptance Checks

- Launch each supported platform in light and dark modes and compare hierarchy, contrast, controls, focus, menus, window dragging, and scrollbars.
- Change the OS theme while C4OS is open; all visible surfaces, focused artifacts, dialogs, popovers, editors, terminal, and Settings update in place.
- Verify no hardcoded light surface or dark text remains in either scheme.
- Verify contrast for primary text, secondary text, borders, focus, selection, disabled controls, status colors, code, and links.
- Verify platform labels, shortcuts, reveal terminology, and window chrome match the host OS.

For macOS implementation, retain the live webview theme listener even if a later Tauri version appears to deliver a native event; treat the signals independently until new accepted evidence deliberately replaces this fallback contract.
