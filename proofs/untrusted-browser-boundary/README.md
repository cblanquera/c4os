# Untrusted Browser Boundary Proof

Status: macOS architecture passed; other targets gated
Updated: 2026-07-18

## Question

Can C4OS render hostile remote content without exposing Tauri/C4OS commands and mediate navigation, popups, profiles, permissions, downloads, and crash recovery?

## Run

```sh
cargo run --manifest-path proofs/native-browser-wry/Cargo.toml
node --test proofs/untrusted-browser-boundary/proof.test.mjs
```

The Wry command launches a native macOS proof window. The controller tests are deterministic and headless.

## Architecture rule

Use a raw-Wry/native isolated webview for untrusted content and do not register a page-accessible Wry IPC handler. `window.ipc` may exist as an unbound object; no Tauri or C4OS bridge may be attached to it. All privileged operations cross a native controller selected outside page JavaScript.

## Result

**Passed for the macOS architecture** on 2026-07-18. The real raw-Wry hostile-page proof passed again, and five controller tests passed for navigation, popup, profile, permission, download, crash, and stale-generation rules.

The permission assertions model C4OS policy behavior; they do not exercise a real camera or microphone prompt. The pinned Wry `0.55.1` predates the merged cross-backend permission callback. Production browser capability acceptance must use a permission-capable release, preserve its `Default` browser/platform behavior, and verify real prompt UX without blanket denial.

Windows/Linux browser enablement remains gated because their webviews were not exercised. Production embedding and individual capabilities retain their own acceptance tests. The prior Tauri `WebviewWindow` failure remains valid; only the raw-Wry/no-IPC architecture is promoted for further implementation.

See `untrusted-browser-boundary-evidence-2026-07-18.md`.
