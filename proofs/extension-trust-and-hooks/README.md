# Extension Trust and Hook Sandbox Proof

Status: macOS technical boundary passed
Updated: 2026-07-18

## Question

Can C4OS verify marketplace origin and content cryptographically, install disabled, revoke compromised releases, and execute an explicitly trusted hook without ambient credentials, network access, or filesystem writes outside its grant?

## Run

```sh
node --test proofs/extension-trust-and-hooks/proof.test.mjs
```

The hook tests require macOS `sandbox-exec`. They run a real Node child process with a generated profile, sanitized environment, denied network, workspace-only writes, bounded output, timeout, and process-group termination.

## Boundaries

The signature fixtures use real Ed25519 verification and local test authorities. They prove the technical trust chain, origin binding, quarantine, explicit enablement, and revocation behavior. Selecting a public C4OS root authority, operating moderation, and cross-platform hook sandboxes remain governance and platform decisions rather than facts this macOS fixture can establish.

## Result

**Passed for the macOS boundary** on 2026-07-18. Four tests passed for cryptographic origin/content verification, disabled installation, explicit trust, revocation, sanitized execution, denied network/outside writes, bounded output, timeout, and process-group termination.

See `extension-trust-and-hooks-evidence-2026-07-18.md`.
