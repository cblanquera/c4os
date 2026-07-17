# OpenCode Runtime Adapter Proof

This proof evaluates OpenCode as a runtime behind a C4OS-owned adapter. It
checks session creation/resume, event streaming, abort behavior, and the
surfaces C4OS would need to wrap for approvals and artifacts.

## Run

The script requires the OpenCode packages recorded in the evidence file to be
available to Node:

```sh
node proofs/opencode-runtime/opencode_runtime_poc.mjs
```

This directory does not declare its own package dependencies. Treat the dated
evidence as a historical dependency-specific result unless those packages are
provided by the active workspace.

## Result

The recorded run found OpenCode viable for session lifecycle, event streaming,
and abort. Model-backed prompting and live permission-request capture remained
partial because the proof intentionally used no provider credentials.

## Limits

C4OS must still own approval records, artifact identity, isolated state paths,
and provider policy around any OpenCode integration.

See `opencode-runtime-evidence-2026-06-20.md` for versions and detailed results.
