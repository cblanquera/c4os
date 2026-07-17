# Raw Wry Native Browser Isolation Proof

This proof evaluates a raw Wry WebView as a narrower C4OS Browser isolation
surface after the Tauri `WebviewWindow` proof exposed Tauri internals.

## Run

From the repository root on a supported desktop environment:

```sh
cargo run --manifest-path proofs/native-browser-wry/Cargo.toml
```

## Result

The recorded macOS run passed its no-Tauri-IPC checks with a warning: page
content could observe Wry's `window.ipc` object, but no handler was registered,
no host command resolved, and no secret leaked.

## Decision

Keep the Wry IPC handler unregistered unless C4OS explicitly accepts a narrow
message boundary. Cross-platform behavior, downloads, sessions, and production
embedding remain unproven.

See `native-browser-wry-evidence-2026-06-20.md` for the recorded checks and raw
result.
