# Tauri Sidecar Packaging Proof

Status: macOS externalBin packaging and ad-hoc integrity passed
Updated: 2026-07-18

## Question

Does a real Tauri 2 bundle include a target-qualified C4OS sidecar, and can the resulting application and sidecar be signed and verified as one macOS bundle?

## Run

```sh
cd proofs/tauri-sidecar-packaging
npm install
rustc binaries/sidecar.rs -o binaries/c4os-sidecar-aarch64-apple-darwin
npm run build
codesign --force --deep --sign - target/release/bundle/macos/C4OS\ Sidecar\ Packaging\ Proof.app
codesign --verify --deep --strict --verbose=2 target/release/bundle/macos/C4OS\ Sidecar\ Packaging\ Proof.app
node --test verification.test.mjs
```

The target-qualified binary must appear inside the application bundle and print `c4os-sidecar-proof 1.0.0` when invoked directly.

## Boundaries

Ad-hoc signing proves bundle integrity and nested-code verification on this macOS host. A distributable Developer ID signature, Apple notarization, Windows signing, Linux packaging, and other-platform descendant cleanup require their real credentials and operating systems.

## Result

Passed for the macOS packaging boundary on 2026-07-18. See `tauri-sidecar-packaging-evidence-2026-07-18.md`.
