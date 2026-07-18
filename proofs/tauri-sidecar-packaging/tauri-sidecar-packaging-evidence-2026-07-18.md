# Tauri Sidecar Packaging Evidence — 2026-07-18

Status: macOS externalBin packaging and ad-hoc bundle integrity passed

## Observed

- Tauri CLI `2.11.4` built a release `.app` from the proof project.
- The target-qualified `c4os-sidecar-aarch64-apple-darwin` source was bundled as `Contents/MacOS/c4os-sidecar` beside the Rust host executable.
- The bundled sidecar executed and printed `c4os-sidecar-proof 1.0.0`.
- `codesign --force --deep --sign -` signed the proof bundle and nested sidecar.
- `codesign --verify --deep --strict --verbose=2` reported the app valid on disk and satisfying its designated requirement.
- The nested sidecar reported `Signature=adhoc` and no Team Identifier.

## Not established

- No Developer ID identity is installed on this host, so distributable signing and Apple notarization were not attempted.
- Windows and Linux packaging/signing and descendant cleanup require those operating systems.
