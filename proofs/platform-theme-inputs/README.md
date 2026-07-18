# Platform Theme Inputs Proof

Status: proved and accepted on macOS; Windows and Linux are `not run` and deferred to a separate spec

## Question

Can C4OS report source-qualified host appearance/accessibility inputs and still produce accessible semantic fallback tokens when values are absent?

## Run

```sh
node --test proofs/platform-theme-inputs/proof.test.mjs
cargo run --manifest-path proofs/platform-theme-inputs/Cargo.toml
```

Review the native AppKit, webview-media, font/control, and semantic-token panels in both macOS appearances. Refresh after changing accessibility preferences.

## Boundaries

The macOS build queries AppKit `NSColor` and `NSWorkspace`. It does not implement Windows `UISettings` or XDG portal access, so those targets are `not run`. Font detection establishes availability to the webview, not licensing or pixel identity. UA controls and system colors remain version-qualified observations.

## Result

Native and webview inputs were source-qualified, and explicit light/dark fallbacks passed all tested contrast checks. See [dated evidence](platform-theme-inputs-evidence-2026-07-18.md).
