# Tauri Native Browser Isolation Proof

This proof evaluates whether a Tauri `WebviewWindow` can provide the isolated
Browser surface C4OS needs while keeping app bridges and secrets unavailable to
page content.

## Run

From the repository root on a supported desktop environment:

```sh
cargo run --manifest-path proofs/native-browser-tauri/Cargo.toml
```

## Result

The recorded macOS run failed the fully unbridged criterion. Navigation and
state reporting worked, but page content could observe
`window.__TAURI_INTERNALS__` even though the secret command was rejected and no
secret leaked.

## Decision

Do not promote this Tauri `WebviewWindow` approach as the C4OS Browser surface
without an accepted IPC-stripping or capability-hardening design. The raw Wry
proof tests the narrower alternative.

See `native-browser-tauri-evidence-2026-06-20.md` for the recorded checks and
raw result.
