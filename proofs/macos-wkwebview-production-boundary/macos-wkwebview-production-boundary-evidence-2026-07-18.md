# macOS Native WebKit Production Boundary Evidence — 2026-07-18

## Environment

- macOS 26.5.1 (25F80), arm64
- Tauri 2.11.2
- `objc2-web-kit` 0.3.2
- System WebKit supplied by the recorded macOS build
- Windows and Linux: `not run`

## Commands

```sh
cargo build --offline --manifest-path proofs/macos-wkwebview-production-boundary/Cargo.toml
cargo run --offline --manifest-path proofs/macos-wkwebview-production-boundary/Cargo.toml
node --test proofs/macos-wkwebview-production-boundary/proof.test.mjs
```

Two consecutive strict native invocations exited successfully after the final harness correction. The retained raw JSON is the final invocation and contains no fixture exception or error sentinel.

## Observed Signal

- All 8 native boundary checks passed.
- The Rust-owned `WKWebView` attached to the Tauri content view, received focus, and matched a real host resize from 900 × 680 to 1040 × 760.
- Eight hostile-page state reports exposed no Tauri global, Tauri internals, Wry IPC, C4OS bridge, WebKit script-message handler, or host secret.
- Navigation allow/block, loading, completion, controlled provisional-navigation error, popup interception, download interception, and process-termination normalization produced sanitized controller events.
- A real camera-and-microphone request reached the public `WKUIDelegate` callback, which returned `WKPermissionDecision::Prompt`. Geolocation remained on WebKit's unmodified system path and returned the recorded platform denial outcome.
- Identifier-backed profiles A and B retained their own cookies, `localStorage`, and IndexedDB values across `WKWebView` recreation without crossing scopes. `sessionStorage` followed page-session lifetime and did not survive recreation.
- A recreated nonpersistent store contained no cookie, `sessionStorage`, `localStorage`, or IndexedDB value.
- Public per-store clearing emptied only profile A. Profile B retained its values. The reliable lifecycle is: clear all public WebKit data types, release the cleared `WKWebsiteDataStore` handle, then reopen the same stable identifier.
- The proof used no private WebKit API, custom raw-profile path, privileged custom scheme, or page-accessible IPC handler.

## Limits

- The controlled connection-close exercised WebKit's real provisional-navigation error delegate.
- The process-termination event exercised the registered callback's deterministic C4OS normalization path; the proof did not forcibly crash the system WebContent process through private API.
- The system permission prompt outcome depends on current macOS device and privacy state. The proof establishes that C4OS preserves `Prompt`; it does not establish user consent.
- This is exact-version feasibility evidence for the macOS boundary, not production integration, packaging, signing, notarization, broad website compatibility, or human acceptance.

## Result

**Passed for the locked macOS public-WebKit boundary.** The result closes Spec 00003's pre-Freeze Browser Proof gate. Production implementation must preserve the proven no-page-IPC boundary, stable/ephemeral profile rules, sanitized controller events, `Prompt` permission behavior, and release-before-reopen scoped-clear lifecycle.

Raw result: [`macos-wkwebview-production-boundary-result.json`](macos-wkwebview-production-boundary-result.json)
