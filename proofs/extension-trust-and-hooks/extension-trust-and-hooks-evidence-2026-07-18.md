# Extension Trust and Hook Sandbox Evidence — 2026-07-18

## Command

```sh
node --test proofs/extension-trust-and-hooks/proof.test.mjs
```

## Observed signal

- 4 tests passed; 0 failed, skipped, cancelled, or todo.
- Real Ed25519 verification bound immutable origin, manifest digest, package digest, and signing key.
- Installation remained disabled and quarantined until explicit enablement.
- Package-digest and signing-key revocation immediately disabled affected content.
- A real macOS `sandbox-exec` child received a sanitized environment and an event payload but no ambient credentials.
- The hook wrote only inside its generated workspace; an outside write and network access were denied.
- Output was bounded, timeout fired, and the isolated process group was killed.

## Result

**Passed for the macOS technical trust and hook-execution boundary.** Public C4OS signing authority, moderation/appeal operations, and Windows/Linux sandbox implementations are separate governance and platform feature gates.
