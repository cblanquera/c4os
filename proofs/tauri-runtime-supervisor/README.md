# Tauri Runtime Supervisor Proof

Status: macOS supervisor boundary passed; cross-platform partial
Updated: 2026-07-18

## Question

Can the C4OS host boundary discover only an explicitly bundled and digest-pinned sidecar, launch it with isolated state and random loopback authentication, enforce version and restart policy, redact logs, and terminate children?

## Run

```sh
node --test proofs/tauri-runtime-supervisor/proof.test.mjs
```

## Scope boundary

The proof exercises the supervisor contract with a real child process, descendant process, and loopback listener. Real macOS `externalBin` packaging and ad-hoc bundle integrity are covered separately by `../tauri-sidecar-packaging/`. It does not claim Developer ID/notarization, Windows/Linux process-tree cleanup, or production OpenCode/Pi binaries.

## Result

**Passed for the macOS supervisor boundary** on 2026-07-18. Four tests passed with real process groups and loopback listeners. The proof rejected unsigned, digest-mismatched, bundle-escaping, and version-mismatched starts; required a random per-launch token; isolated state roots; bounded restarts; redacted logs; and observed parent and descendant exit after shutdown.

Windows/Linux descendant-process cleanup and distributable platform signing remain not proven.

See `tauri-runtime-supervisor-evidence-2026-07-18.md`.
