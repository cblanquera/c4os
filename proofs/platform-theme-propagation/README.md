# Platform Theme Propagation Proof

Status: accepted macOS fallback; Windows and Linux are `not run` and deferred to a separate spec

## Question

Can a minimal Tauri shell identify the platform, choose the correct initial system scheme before reveal, follow live system changes, and preserve fixture state?

## Run

```sh
node --test proofs/platform-theme-propagation/proof.test.mjs
cargo run --manifest-path proofs/platform-theme-propagation/Cargo.toml
```

With the native window open, change the macOS Appearance between Light and Dark. Confirm:

- native and webview signals agree after each change;
- the root scheme updates without reload;
- the draft input and counter retain their values;
- the signal timeline records native and webview events independently;
- no wrong-scheme frame is visible during launch.

## Boundaries

The static test checks source ordering and fallback presence. Native event delivery and first-frame behavior require visual execution. Windows and Linux remain `not run` on this host and must not inherit the macOS result.

## Result

The macOS webview updated live and retained state, but Tauri's native theme-change event did not fire. The user accepted the independent live webview listener as the required fallback. See [dated evidence](platform-theme-propagation-evidence-2026-07-18.md).
