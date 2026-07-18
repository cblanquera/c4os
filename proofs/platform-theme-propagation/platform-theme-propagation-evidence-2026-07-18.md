# Platform Theme Propagation Evidence — 2026-07-18

## Environment

- macOS 26.5.1 (25F80), arm64
- Tauri 2.11.5, tao 0.35.3, wry 0.55.1
- WKWebView user agent: `AppleWebKit/605.1.15`
- Windows and Linux: `not run`

## Commands

```sh
node --test proofs/platform-theme-propagation/proof.test.mjs
cargo check --manifest-path proofs/platform-theme-propagation/Cargo.toml
```

The release `.app` bundle was also built and exercised as a native macOS application.

## Observed Signal

- 3 static contract tests passed.
- The application launched in the active Dark appearance with native snapshot, webview media query, and root semantic state agreeing.
- Changing macOS Appearance from Dark to Light updated the visible webview live without navigation or reload.
- The edited draft value `State retained after live theme change` and counter value `8` survived the live change.
- A fresh native snapshot after the change reported Light and agreed with the webview.
- No wrong-theme frame was visible during the observed launch.
- Tauri's native `WindowEvent::ThemeChanged` did not appear in the event timeline. The independent webview `prefers-color-scheme` listener did fire.

## Result

**Partial on macOS.** Startup resolution, live webview propagation, fresh native snapshots, semantic updating, and state retention passed. Native live-event delivery did not, so production behavior must retain the webview listener as an independent live authority/fallback and must not rely exclusively on `WindowEvent::ThemeChanged`. First-frame absence of a flash is a visual observation from this run, not instrumented frame-timing proof. Windows and Linux remain `not run`.
