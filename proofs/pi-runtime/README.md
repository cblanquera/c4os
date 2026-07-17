# Pi Runtime Adapter Proof

This proof evaluates Pi as an in-process runtime behind a C4OS-owned adapter.
It checks session lifecycle, a faux-provider prompt, streamed events, tool-call
interception, denial before execution, and abort support.

## Run

The script requires the Pi packages recorded in the evidence file to be
available to Node:

```sh
node proofs/pi-runtime/pi_runtime_poc.mjs
```

This directory does not declare its own package dependencies. Treat the dated
evidence as a historical dependency-specific result unless those packages are
provided by the active workspace.

## Result

The recorded run passed the tested session, streaming, interception, denial,
and cancellation boundaries. Artifact identity remained partial and belongs to
the C4OS application layer.

## Limits

Pi does not supply the complete C4OS security boundary. C4OS must own sandbox,
approval, persistence, trusted-root, provider, and secret-storage policy.

See `pi-runtime-evidence-2026-06-20.md` for versions and detailed results.
