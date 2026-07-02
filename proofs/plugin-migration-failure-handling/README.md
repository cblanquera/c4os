# Plugin Migration Failure Handling Proof

Status: proof artifact
Updated: 2026-07-02

## Run

```sh
node --test proofs/plugin-migration-failure-handling/proof.test.mjs
```

## Result

The harness proves representative migration failures route through the
spec-local classes: cache auto-recovery, corrupt user config reset while kept
enabled, incompatible schema disablement, dependency-blocked state, and
security disablement.
