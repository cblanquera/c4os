# Research Plan And Findings

State: Bounded primary-source Research Loop complete and accepted for macOS Freeze 2026-07-18; Windows/Linux native work deferred to a separate spec.

## Research Boundary

- Sources were limited to current Tauri documentation/source references, platform-vendor documentation, W3C specifications, WebKit, WebKitGTK/GTK, and XDG specifications.
- Documentation establishes API availability and intended semantics. It does not establish first-frame behavior, event delivery, computed webview styling, or native window behavior on an actual target.
- Native evidence is macOS 26.5.1 on arm64 using Tauri 2.11.5 and the host WKWebView. P-001 through P-004 were created and run; Windows/Linux were not run.

## Topic Results

### R-001 — Tauri theme and platform APIs

State: Research complete; macOS P-001 executed with an accepted fallback; Windows/Linux deferred.

- Tauri 2 exposes `theme()` for the current window theme, `onThemeChanged()` for system-theme changes, and `setTheme(null | undefined)` to resume following the system.
- Initial `WindowOptions.theme` defaults to the system theme but is documented as implemented only on Windows and macOS 10.14+. On Linux and macOS, an explicit `setTheme` is app-wide rather than window-specific.
- Tauri can create a window with `visible: false`; its own window-state guidance uses this to prevent a restore-time flash. This makes hidden initialization credible but does not prove a flash-free C4OS first frame.
- Tauri emits a `ThemeChanged` window event, while its underlying window-event documentation qualifies native theme-change support by platform. Therefore C4OS must not assume identical event delivery on macOS and Linux without P-001.
- Resolution boundary: keep the native window theme unset, read native theme when it returns a value, expose the platform and resolved scheme to the document, and retain `matchMedia('(prefers-color-scheme: dark)')` as a live webview fallback.

### R-002 — Webview theme behavior

State: Research complete; macOS P-001/P-002 executed; Windows/Linux deferred.

- Tauri uses WKWebView on macOS, WebView2 on Windows, and WebKitGTK on Linux. Their versions are supplied by the host rather than bundled uniformly by Tauri.
- W3C `color-scheme` negotiation affects browser-controlled backgrounds, form controls, scrollbars, and CSS system colors. The HTML color-scheme metadata can advertise supported schemes before the stylesheet is fully processed.
- WebKit documents `color-scheme: light dark`, the color-scheme metadata, `prefers-color-scheme`, adaptive form controls, scrollbars, and system colors. It explicitly warns against assuming fixed system-color values.
- WebView2's profile color scheme defaults to `Auto`, which follows the OS. Windows forced-colors behavior can replace author colors and shadows, so C4OS must preserve CSS system colors and avoid broad `forced-color-adjust: none` rules.
- WebKitGTK has no Tauri-level guarantee that makes its exact form-control, scrollbar, forced-color, or startup behavior interchangeable with WKWebView. Those claims remain target-qualified until P-001/P-002.

### R-003 — Host accessibility and appearance inputs

State: Research complete; macOS P-002 executed; Windows/Linux deferred.

- macOS AppKit inherits the system appearance when the app does not set one. `NSWorkspace` exposes increase-contrast, reduce-motion, reduce-transparency, invert-colors, and differentiate-without-color preferences plus a change notification.
- Windows `UISettings` exposes system colors, animation, advanced effects, auto-hide scrollbars, text scaling, and change events. `AccessibilitySettings` exposes high contrast and a change event.
- XDG Settings portal version 2 standardizes color scheme, sRGB accent color, higher contrast, reduced motion, and `SettingChanged`. Unknown values must be treated as no preference.
- CSS preference media features remain the lowest-coupling content inputs for scheme, contrast, forced colors, and reduced motion. Native values may enrich diagnostics and semantic-token construction but must retain source provenance.

### R-004 — Fonts and platform metrics

State: Research complete; macOS P-002 executed; Windows/Linux deferred.

- CSS Fonts defines `system-ui` as the system UI font and provides `ui-monospace` as a platform-resolved generic when the system supplies one. Named family stacks may precede these generics, but generic fallbacks are required.
- GTK exposes `gtk-font-name`, animation, decoration-layout, overlay-scrolling, and newer interface preference properties. These are GTK-specific, version-dependent inputs, not a Linux-wide contract.
- XDG Settings portal does not standardize an active desktop font, radii, control metrics, scrollbars, or decoration layout.
- Proposed fallback: use `system-ui, sans-serif` for UI and `ui-monospace, monospace` for code. Treat named macOS/Windows/Linux families and GTK font queries as optional refinements, never release requirements.

### R-005 — Native menus and window chrome

State: Research complete; macOS P-003 baseline proved; Windows/Linux deferred.

- Tauri native menus render in the global menu bar on macOS and in the application window on Windows/Linux. macOS requires top-level submenus and treats the first submenu as the application menu.
- Tauri supports menu accelerators and predefined native behaviors, but Settings is a custom application action. Proposed shortcuts are `Cmd+,` on macOS and `Ctrl+,` on Windows/Linux, subject to P-003.
- Standard decorated windows preserve the strongest native behavior and are the required baseline. Tauri warns that undecorated custom titlebars on macOS lose system moving/alignment behavior.
- Windows guidance requires titlebar drag, double-click maximize/restore, right-click system window menu, visible caption states, and light/dark/high-contrast adaptation. A custom drag strip alone does not establish these behaviors.
- Transparent/custom titlebars, moved traffic lights, materials, and window effects are optional target capabilities. They must not block the neutral standard-decoration fallback.

### R-006 — Supported target and acceptance matrix

State: Research complete; macOS target accepted, Windows/Linux native targets deferred to a separate spec.

Tauri's engineering floor is broader than a defensible C4OS release claim: its prerequisites list macOS 10.15+ and Windows 7+, while Linux compatibility depends on distribution libraries, WebKitGTK 4.1, packaging baseline, desktop, compositor, and portal backend. The candidate proof matrix is deliberately narrower:

| Claim | Candidate named target | Required native evidence |
| --- | --- | --- |
| macOS primary | macOS 26.5.x, arm64, WKWebView; current local host | P-001 through P-004 |
| Windows primary | Windows 11 25H2, x64, Evergreen WebView2 with version recorded | P-001 through P-004 |
| Linux primary | Ubuntu 24.04 LTS, x64, GNOME on Wayland, WebKitGTK 4.1 and portal backend recorded | P-001 through P-004 |
| Linux variation | KDE Plasma on Wayland on an accepted supported distribution | P-001 through P-003 before any KDE claim; P-004 for visual support |
| X11 compatibility | GNOME or KDE on X11 on an accepted supported distribution | `not run` and gated unless explicitly added |

- macOS 26.5.x is chosen because it is the available native host, not as a minimum-version promise.
- Windows 25H2 is a current generally available release with support through 2027-10-12 for Home/Pro; 26H1 is device-scoped rather than a general in-place feature update.
- Ubuntu 24.04 LTS remains in standard security maintenance through May 2029. Tauri also names Ubuntu 22.04 as a suitable older AppImage build baseline, but build compatibility is not visual QA coverage.
- CPU architecture is part of a target claim. arm64 Windows/Linux and x64 macOS remain gated until explicitly added and tested.

## Build And Evidence Process By OS

This process separates successful compilation and automated checks from native visual acceptance. A CI pass never upgrades an unavailable interactive target to visually supported.

### Shared build record

For every build, record the commit, OS release/build, CPU architecture, Rust toolchain, Node/package-manager versions, Tauri CLI and crate versions, webview version, build command, test commands, and produced artifact. Linux additionally records distribution, desktop, compositor, display protocol, WebKitGTK version, and portal backend.

### macOS

1. Build and run directly on the available macOS arm64 host using its WKWebView.
2. Execute P-001 through P-003 interactively, including launch capture, live system-theme changes, menus, shortcuts, focus, dragging, resizing, and standard decorations.
3. Execute P-004 only after those platform behaviors pass; preserve light/dark and accessibility screenshots with the build record.
4. Label the result `macOS arm64 native`. It establishes no macOS x64 or earlier-version claim.

### Windows

1. Use a GitHub-hosted `windows-latest` x64 runner to compile, run Rust/unit checks, and run Tauri WebDriver checks where implemented.
2. Preserve the runner image, Windows build, architecture, WebView2 version, logs, and application artifact. Label this `Windows x64 CI`, not native visual acceptance.
3. For interactive evidence, either run Windows 11 ARM locally in an Apple-silicon VM or use an interactive Windows x64 cloud/borrowed machine.
4. A local ARM VM may execute P-001 through P-004, but its result is `Windows arm64 native`; it does not satisfy the Windows x64 claim even when Windows emulates x64 applications.
5. Keep Windows x64 visual acceptance `not run` until P-001 through P-004 execute on an interactive x64 target.

### Linux

1. Use a GitHub-hosted `ubuntu-24.04` x64 runner to compile, run Rust/unit checks, and run Tauri WebDriver checks under the CI virtual display.
2. Preserve the runner image, architecture, WebKitGTK and portal package versions, logs, and application artifact. Label this `Ubuntu x64 CI`, not GNOME/Wayland visual acceptance.
3. For interactive evidence, either run an Ubuntu ARM64 desktop VM locally or use an interactive Linux x64 cloud/borrowed machine with the named desktop and display protocol.
4. A local ARM64 VM may execute P-001 through P-004, but its result is limited to that exact distribution, desktop, compositor, display protocol, portal backend, WebKitGTK version, and architecture.
5. Keep Ubuntu x64 GNOME/Wayland, KDE/Wayland, and X11 visual claims `not run` until each exact interactive target is exercised.

### Accepted closeout sequence

1. Native macOS arm64 P-001 through P-004 are complete and accepted with the documented P-001 fallback.
2. Windows and Linux builds, interactive checks, release claims, and platform variations are deferred to a separate spec.
3. No CI, simulation, VM, or macOS result is promoted as Windows/Linux native visual evidence.

## Resulting Implementation Contract For Proofs

1. Leave the native window theme unset so the host remains authoritative.
2. Resolve a source-qualified scheme from native theme when available, then webview media query, then a bounded neutral fallback.
3. Advertise `light dark` before first paint, set semantic tokens on the root, and never persist a C4OS-only override.
4. Normalize only documented inputs. Every unavailable or invalid accent, contrast, motion, font, control, or scrollbar value uses an explicit semantic fallback.
5. Start P-003 with native menus and standard decorations. Test optional chrome only after the baseline passes.
6. Record OS, architecture, Tauri, webview, desktop, compositor, portal backend, and preference mode beside every result.

## Primary Sources

Accessed 2026-07-18:

- [Tauri Window API](https://v2.tauri.app/reference/javascript/api/namespacewindow/)
- [Tauri Window Customization](https://v2.tauri.app/learn/window-customization/)
- [Tauri Window Menu](https://v2.tauri.app/learn/window-menu/)
- [Tauri Window State](https://v2.tauri.app/plugin/window-state/)
- [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/)
- [Tauri webview versions](https://v2.tauri.app/reference/webview-versions/)
- [Tauri AppImage limitations](https://v2.tauri.app/distribute/appimage/)
- [Tauri Rust `WindowEvent`](https://docs.rs/tauri/latest/tauri/enum.WindowEvent.html)
- [W3C CSS Color Adjustment Level 1](https://www.w3.org/TR/css-color-adjust-1/)
- [W3C CSS Color Level 4](https://www.w3.org/TR/css-color-4/)
- [W3C CSS Fonts Level 4](https://www.w3.org/TR/css-fonts-4/)
- [W3C Media Queries Level 5](https://www.w3.org/TR/mediaqueries-5/)
- [WebKit dark-mode support](https://webkit.org/blog/8840/dark-mode-support-in-webkit/)
- [Microsoft WebView2 preferred color scheme](https://learn.microsoft.com/en-us/microsoft-edge/webview2/reference/winrt/microsoft_web_webview2_core/corewebview2preferredcolorscheme)
- [Microsoft `UISettings`](https://learn.microsoft.com/en-us/uwp/api/windows.ui.viewmanagement.uisettings)
- [Microsoft `AccessibilitySettings`](https://learn.microsoft.com/en-us/uwp/api/windows.ui.viewmanagement.accessibilitysettings)
- [Windows titlebar guidance](https://learn.microsoft.com/en-us/windows/apps/design/basics/titlebar-design)
- [Apple `NSAppearance`](https://developer.apple.com/documentation/appkit/nsappearance)
- [Apple accessibility display preferences](https://developer.apple.com/documentation/appkit/nsworkspace/accessibilitydisplayshouldincreasecontrast)
- [XDG Settings portal](https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.Settings.html)
- [GTK Settings](https://docs.gtk.org/gtk4/class.Settings.html)
- [Ubuntu release cycle](https://ubuntu.com/about/release-cycle)
- [Windows 11 lifecycle](https://learn.microsoft.com/en-us/lifecycle/products/windows-11-home-and-pro)
