# Research Plan

State: Planned; full Research Loop not started

## Pre-Spec Reconnaissance

The read-only KB audit on 2026-07-18 established enough direction to justify this spec, but it did not complete target research:

- Tauri's OS plugin exposes compile-time platform identity including `macos`, `windows`, and `linux`.
- Tauri exposes system theme-change events and JavaScript listeners; exact delivery and window-theme requirements must be verified per target.
- Tauri supports desktop native menus, with platform-specific presentation rules.
- CSS `color-scheme`, early color-scheme metadata, and `prefers-color-scheme` provide a credible webview baseline.
- The standardized XDG Settings portal exposes color scheme, accent, contrast, and reduced-motion settings, but not a standardized active desktop font.
- Tauri documents tradeoffs and platform limits for custom or transparent titlebars.

These are starting leads, not accepted feasibility results.

## Topics

| ID | Topic | Questions | Affected Gaps | State |
| --- | --- | --- | --- | --- |
| R-001 | Tauri theme and platform APIs | Initial theme, `ThemeChanged`, `onThemeChanged`, unset/follow-system behavior, window/app-wide scope, platform plugin, event cleanup | GAP-001, GAP-002 | Planned |
| R-002 | Webview theme behavior | WKWebView, WebView2, and WebKitGTK support for `color-scheme`, `prefers-color-scheme`, CSS system colors, form controls, scrollbars, forced colors, and startup metadata | GAP-001–GAP-003, GAP-005 | Planned |
| R-003 | Host accessibility and appearance inputs | macOS/Windows APIs and XDG portals for accent, high contrast, reduced motion, transparency/material preferences, and reliable fallback behavior | GAP-003, GAP-005 | Planned |
| R-004 | Fonts and platform metrics | Availability and licensing of UI/mono font stacks; whether active Linux desktop fonts, radii, or control metrics have stable APIs | GAP-003, GAP-005 | Planned |
| R-005 | Native menus and window chrome | Application menus, Settings shortcuts, standard/transparent/custom titlebars, traffic-light/caption insets, drag regions, focus, native window menu behavior | GAP-004, GAP-005 | Planned |
| R-006 | Supported target and acceptance matrix | Candidate OS versions, GNOME/KDE/Wayland/X11 scope, screenshot and event evidence, contrast/reduced-motion/high-contrast checks | GAP-005, GAP-006 | Planned |

## Source Rules

- Prefer current official Tauri documentation and source, platform-vendor documentation, WebKit/WebView2 documentation, web standards, and XDG specifications.
- Use issues only as bounded evidence of a reproducible limitation, never as a stable API contract.
- Record access dates, exact versions, platform qualifications, and contradictions.
- Do not infer Linux-wide behavior from one distribution or desktop environment.

## Starting Sources

- [Tauri OS Information](https://v2.tauri.app/plugin/os-info/)
- [Tauri Window API](https://v2.tauri.app/reference/javascript/api/namespacewindow/)
- [Tauri Window Customization](https://v2.tauri.app/learn/window-customization/)
- [Tauri Window Menu](https://v2.tauri.app/learn/window-menu/)
- [MDN color-scheme](https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/Properties/color-scheme)
- [MDN prefers-color-scheme](https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/At-rules/%40media/prefers-color-scheme)
- [XDG Settings portal](https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.Settings.html)
