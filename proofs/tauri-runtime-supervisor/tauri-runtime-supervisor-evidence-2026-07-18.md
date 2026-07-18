# Tauri Runtime Supervisor Evidence — 2026-07-18

## Command

```sh
node --test proofs/tauri-runtime-supervisor/proof.test.mjs
```

Loopback binding required running outside the restricted network sandbox.

## Observed signal

- 4 tests passed; 0 failed, skipped, cancelled, or todo.
- Bundled-path canonicalization and SHA-256 verification passed.
- Unsigned, tampered, and version-mismatched inputs were rejected.
- Two simultaneous launches used distinct random tokens and state roots.
- Missing/wrong tokens returned 401; the correct token returned healthy state/version data.
- The restart ceiling entered degraded state after the configured limit.
- Token/password/Bearer values were redacted.
- Spawned parent and descendant processes exited after supervisor shutdown through the macOS process-group boundary.

## Result

**Passed for the macOS supervisor boundary.** Real Tauri `externalBin` packaging and ad-hoc bundle verification also passed in `../tauri-sidecar-packaging/`. Developer ID/notarization and Windows/Linux process-tree behavior remain target-specific release gates.
