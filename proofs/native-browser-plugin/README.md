# Native Browser Plugin Proof

This loopback-browser proof checks the C4OS Browser isolation model before any
production Browser integration. It exercises navigation policy, host/page
separation, constrained messages, and observable reload/stop state.

## Run

The script requires Playwright and a compatible Chromium installation to be
available to Node:

```sh
node proofs/native-browser-plugin/isolated-browser-poc.mjs
```

This directory does not declare its own package dependencies. If Playwright is
not already available in the workspace, use the recorded evidence rather than
assuming the proof is immediately reproducible.

## Result

The recorded run passed. The isolated page could not access the host bridge or
secret sentinel, and the navigation policy rejected privileged URL schemes.

## Limits

The proof uses loopback pages instead of live internet navigation. It does not
establish persistent sessions, downloads, cross-platform native WebView
behavior, or production Browser integration.

See `native-browser-plugin-evidence-2026-06-20.md` for the recorded checks and
raw result.
