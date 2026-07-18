# Untrusted Browser Boundary Evidence — 2026-07-18

## Commands

```sh
cargo run --manifest-path proofs/native-browser-wry/Cargo.toml
node --test proofs/untrusted-browser-boundary/proof.test.mjs
```

## Observed signal

- Raw Wry native proof: passed on macOS.
- Hostile page observed no `window.__TAURI__`, `window.__TAURI_INTERNALS__`, C4OS bridge, secret, shell state, opener, or callable host command.
- Wry's `window.ipc` object was visible but unbound.
- Privileged schemes and new-window requests were blocked.
- 5 controller tests passed; 0 failed, skipped, cancelled, or todo.
- Permissions were user-approved, origin/profile-bound, and single-use.
- Downloads required policy, stayed inside a granted root, and used exact single-use authorization.
- Private and persistent profiles did not share cookies.
- Crash cleared transient cookies/permissions/download authorization and rejected stale-generation events.

## Result

**Passed for the macOS raw-Wry isolation architecture.** Do not use a normal Tauri `WebviewWindow` for arbitrary remote content. Do not register a page-accessible Wry IPC handler. The controller permission checks are modeled policy evidence, not a real camera/microphone prompt. Windows/Linux and each production capability remain target- or feature-gated until equivalent evidence exists, with normal platform/browser permission behavior preserved.
