# Marketplace Trust and Rollback Evidence — 2026-07-18

## Command

```sh
node --test proofs/marketplace-trust-and-rollback/proof.test.mjs
```

## Observed signal

- 5 tests passed; 0 failed, skipped, cancelled, or todo.
- Mutable Git refs and npm version ranges were rejected.
- A mismatched SHA-256 digest prevented installation.
- Metadata review preceded C4OS-cache write; install state was disabled.
- A package activation fixture was never executed.
- Failed migration removed staging and preserved the enabled 1.0.0 record.
- Successful 2.0.0 update committed disabled.
- Revoked content could not be re-enabled.
- Uninstall removed the executable cache path and install record.

## Result

**Passed for the local transactional lifecycle.** Remote registries, public signing authorities, moderation, and service availability remain outside the claim.
