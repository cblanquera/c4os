# Platform Window And Menu Evidence — 2026-07-18

## Environment

- macOS 26.5.1 (25F80), arm64
- Tauri 2.11.5, tao 0.35.3, wry 0.55.1
- WKWebView user agent family: `AppleWebKit/605.1.15`
- Windows and Linux: `not run`

## Commands

```sh
node --test proofs/platform-window-menu/proof.test.mjs
cargo check --manifest-path proofs/platform-window-menu/Cargo.toml
```

The release `.app` bundle was built and exercised as a native macOS application.

## Observed Signal

- 3 static native-menu and routing contract tests passed.
- The window used the standard native titlebar and traffic lights; no fake caption controls or content titlebar were present.
- The native application menu exposed `Settings…`.
- Activating that native menu item opened the owned in-app Settings dialog and recorded `Settings opened from native-menu`.
- `Cmd+,` opened the same Settings route.
- Initial focus entered the dialog, Escape dismissed it, and the standard macOS window controls remained available.
- The proof deliberately retained standard decorations; transparent, overlay, and custom titlebars were not evaluated.

## Result

**Passed for the macOS standard-decoration baseline.** Native menu routing and the macOS Settings shortcut work without imitating native controls inside web content. Windows caption/system-menu behavior and Linux compositor/window-manager behavior remain `not run`.
