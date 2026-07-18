# macOS Native WebKit Production Boundary Proof

Status: passed 2026-07-18; exact-version feasibility evidence only

## Question

Can a public-API native `WKWebView` be embedded as a child of the locked Tauri 2 macOS application while keeping hostile websites outside Tauri/Wry/C4OS IPC, preserving WebKit permission behavior, and enforcing the accepted persistent and ephemeral Browser Environment lifecycle?

## Gap And Decision

- Gap: Spec 00003 `GAP-005`.
- Selection: Spec 00003 `D-023` and `IS-003`.
- Target: current macOS development environment only.
- Locked proof baselines: Tauri `2.11.2`, `objc2-web-kit` `0.3.2`, and the system WebKit shipped with the recorded macOS build.

The direct WebKit boundary is necessary because Tauri `WebviewWindow` exposes Tauri internals to its page, while Wry `0.55.1` has no public permission-policy handler and its macOS UI delegate answers media capture with `WKPermissionDecision::Grant`. C4OS requires a controller that can return WebKit's `Prompt` decision without exposing a page command bridge.

## Hypothesis

A Rust-owned native `WKWebView` child/controller can:

- attach, resize, and focus inside a Tauri application window;
- expose no Tauri, Wry, C4OS, custom-scheme, or script-message command bridge to website JavaScript;
- allow only sanitized controller events to cross into Rust;
- mediate navigation, popups, and downloads outside page JavaScript;
- exercise media and geolocation requests through WebKit/system behavior without blanket grant or denial;
- isolate stable identifier-backed persistent profiles, discard nonpersistent profiles on close, and clear one persistent scope without affecting another.

## Expected Signal

The recorded native run and deterministic assertions pass all of the following:

1. the WebKit child is attached to the Tauri content view and follows a real host-window resize;
2. a hostile loopback page cannot observe or invoke a host bridge or receive secret sentinels;
3. allowed navigation, blocked privileged schemes, popups, downloads, loading, errors, and process-termination callbacks are represented by sanitized controller events;
4. a real page media request reaches the public `WKUIDelegate` callback and the controller returns `WKPermissionDecision::Prompt`; a page geolocation request remains on the unmodified WebKit/system path;
5. two identifier-backed profiles remain isolated and survive `WKWebView` recreation;
6. a fresh nonpersistent profile restores no cookies, `localStorage`, `sessionStorage`, or IndexedDB values;
7. clearing all public WebKit data types through one identifier-backed store empties only that profile;
8. no private API, custom raw-profile path, page IPC handler, or privileged custom scheme is used.

## Failure Signal

Fail if any page-accessible host IPC or secret appears, a controller event contains unsanitized page data, profile data crosses scopes, ephemeral data survives recreation, scoped clearing affects another profile, the media delegate cannot preserve `Prompt`, native child embedding/resizing is unstable, or the implementation requires private API or a custom raw-profile path.

## Scope

- One visible Tauri host window and one replaceable native `WKWebView` child.
- Hostile loopback HTTP fixtures and page-origin storage.
- Cookies, `sessionStorage`, `localStorage`, and IndexedDB.
- Identifier-backed and nonpersistent `WKWebsiteDataStore` instances.
- Navigation, popup, download, loading/error, media-permission, geolocation-attempt, and crash-callback controller paths.
- Raw JSON evidence plus a dated Markdown evidence record.

## Non-Goals

- Production Browser UI or product code.
- Browser sub-tabs or arbitrary automation.
- Windows/Linux webviews.
- Distribution signing, notarization, or App Store packaging.
- Proving every public website or hardware permission outcome.
- Replacing production integration, permission-copy, entitlement, or human-acceptance tests.

## Run

```sh
cargo build --offline --manifest-path proofs/macos-wkwebview-production-boundary/Cargo.toml
cargo run --offline --manifest-path proofs/macos-wkwebview-production-boundary/Cargo.toml
node --test proofs/macos-wkwebview-production-boundary/proof.test.mjs
```

The native command opens a short-lived macOS window. The page requests media and geolocation access to exercise the real public WebKit path; device/system outcomes are recorded but are not forced or treated as user consent.

## Result

Passed twice consecutively on macOS 26.5.1 (25F80), arm64, with Tauri 2.11.2 and `objc2-web-kit` 0.3.2. All eight boundary checks passed and the final raw result contains no fixture exception or error sentinel.

See [the dated evidence record](macos-wkwebview-production-boundary-evidence-2026-07-18.md) and [the raw final result](macos-wkwebview-production-boundary-result.json). The proof closes the pre-Freeze feasibility gate; it does not replace production integration, target, release, or human-acceptance verification.
