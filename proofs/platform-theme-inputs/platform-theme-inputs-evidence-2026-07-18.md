# Platform Theme Inputs Evidence — 2026-07-18

## Environment

- macOS 26.5.1 (25F80), arm64
- Tauri 2.11.5, tao 0.35.3, wry 0.55.1
- WKWebView user agent: `Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko)`
- Windows and Linux: `not run`

## Commands

```sh
node --test proofs/platform-theme-inputs/proof.test.mjs
cargo check --manifest-path proofs/platform-theme-inputs/Cargo.toml
```

The release `.app` bundle was built and reviewed in macOS Light and Dark.

## Observed Signal

- 4 static token and source-contract tests passed.
- AppKit reported the current scheme, macOS accent `rgb(0, 122, 255)`, and false for increase contrast, reduce motion, reduce transparency, differentiate without color, and invert colors on this machine.
- Webview media queries independently reported the scheme and preference flags.
- `system-ui`, `ui-monospace`, `-apple-system`, `SF Mono`, and `Menlo` were available to this webview.
- Light fallback contrast ratios passed: primary 16.77:1, secondary 6.67:1, accent 5.67:1, danger 6.14:1.
- Dark fallback contrast ratios passed: primary 15.65:1, secondary 8.86:1, accent 7.23:1, danger 7.47:1.
- UA system colors and native controls followed the active macOS appearance.

## Result

**Passed on macOS.** The proof distinguishes native AppKit, webview, font/UA, and explicit semantic-fallback sources, and both fallback schemes meet the tested 4.5:1 threshold. The observations are version-qualified. Windows `UISettings`, XDG portals, WebKitGTK, Linux fonts, and Linux toolkit/control behavior remain `not run`.
