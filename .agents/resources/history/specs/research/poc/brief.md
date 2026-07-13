# Research POC Brief

Status: planned
Updated: 2026-06-20

This folder contains proof planning and results for research-stage MVP feasibility questions.
Runnable proof implementations should live under `proofs/<proof-name>/` and
link back here.

## POC-001: Runtime Adapter Target

Status: planned
Confidence: proposed
MVP: yes
Phase: poc
Source: `.agents/specs/research/questions.md`
Related:
- Q-001
- ASM-001
- RISK-001
- TASK-002

### Objective

Decide whether C4OS should target OpenCode Runtime, Pi Runtime, or a thin
runtime adapter that can support both before the MVP scope is frozen.

### Proof Tracks

- OpenCode runtime proof: `proofs/opencode-runtime/`
- Pi runtime proof: `proofs/pi-runtime/`

Each proof should use the same small adapter contract so the comparison is
behavioral, not just a demo of each runtime's happy path.

### Questions To Answer

- Can the runtime create and resume a project-scoped session from a trusted
  local folder?
- Can it stream assistant output and tool/runtime events in a shape the app can
  store as app-owned records?
- Can proposed shell, file, network, MCP, or credential-sensitive actions be
  intercepted before execution by app-owned approval policy?
- Can provider/model configuration be supplied without leaking raw secrets into
  workspace descriptors, transcripts, or logs?
- Can the app stop, cancel, or recover from a run without orphaned process
  state?
- Can runtime-specific concepts stay behind an adapter boundary so the product
  model remains C4OS-owned?

### Shared Adapter Contract To Prove

- `createSession({ trustedRoot, providerRef, modelRef, instructionsRef })`
- `resumeSession({ sessionId })`
- `sendUserMessage({ sessionId, content })`
- `streamEvents({ sessionId })`
- `requestApproval({ sessionId, action })`
- `resolveApproval({ approvalId, decision })`
- `stopRun({ sessionId, runId })`
- `listArtifacts({ sessionId })`

The proof can use a smaller implementation internally, but results should map
observed runtime behavior back to this contract.

### Expected Proof

- A minimal script or harness starts one session in a fixture project.
- The harness sends one prompt that triggers at least one observable runtime
  event stream.
- The harness captures one sensitive action proposal or documents why the
  runtime cannot expose one before execution.
- The harness records stop/cancel behavior.
- The output includes a short evidence note under the proof directory with
  commands run, observed behavior, gaps, and recommended runtime direction.

### Failure Signals

- No stable programmatic session/run control surface is available.
- Tool or shell actions cannot be intercepted before execution.
- Event streaming cannot be mapped into app-owned session records.
- Provider/model configuration requires unsafe secret handling.
- Stop/cancel leaves unmanaged runtime state.
- Runtime concepts force product-visible coupling that contradicts RISK-001.

### Verification Method

Run each proof from a clean local checkout with isolated temp/config paths.
Record the command, environment assumptions, output summary, and observed
pass/fail results in:

- `proofs/opencode-runtime/opencode-runtime-evidence-YYYY-MM-DD.md`
- `proofs/pi-runtime/pi-runtime-evidence-YYYY-MM-DD.md`

### Decisions Unlocked

- Accept OpenCode Runtime as the first MVP runtime target.
- Accept Pi Runtime as the first MVP runtime target.
- Build a thin adapter and keep both as supported targets after MVP.
- Reject one or both runtimes for MVP and revise the architecture before
  freeze.

### Estimated Effort

One focused proof session per runtime, plus one comparison pass.
