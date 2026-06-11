# FINDING-014 Runtime Fallback Decision

Date: 2026-06-10

## Question

What is the accepted runtime path now that direct OpenCode failed the strict
runtime config isolation gate?

## Evidence Used

- `.agents/archive/planning-corpus/validation/FINDING-001-opencode-integration-surface.md`
- `.agents/archive/planning-corpus/validation/FINDING-001-structured-events.md`
- `.agents/archive/planning-corpus/validation/FINDING-001-pre-execution-interception.md`
- `.agents/archive/planning-corpus/validation/FINDING-001-stop-behavior.md`
- `.agents/archive/planning-corpus/validation/FINDING-001-app-owned-record-mapping.md`
- `.agents/archive/planning-corpus/validation/FINDING-001-runtime-config-isolation.md`
- `.agents/archive/planning-corpus/validation/FINDING-001-instruction-loading-observability.md`

## Current Runtime Evidence

| Gate | Result | Impact |
| --- | --- | --- |
| Integration surface | Passed enough to continue | Direct API/SDK/SSE integration exists. |
| Structured events | Passed | Terminal scraping is not required for the tested paths. |
| Pre-execution interception | Passed | Shell, file-write, and Git side effects waited for permission in tested paths. |
| Structured denial | Passed | Reject produced structured runtime state. |
| Stop behavior | Passed with adapter caveat | Abort terminates the tested child process; adapter must map abort metadata to `stopped`. |
| App-owned record mapping | Passed | OpenCode IDs can remain adapter refs. |
| Instruction-loading observability | Passed with disclosure policy | Native instruction loading is not disabled, but can be disclosed through app preflight plus `/config`. |
| Runtime config isolation | Failed | Project config instructions still merge even with `--pure` and app `instructions: []`. |

## Non-Negotiable MVP Runtime Gates

These cannot be weakened without invalidating the MVP safety model:

- Protected file writes do not execute before backend approval.
- Shell commands do not execute before backend approval.
- Runtime-proposed Git state changes do not execute before backend approval.
- Denials return structured runtime state.
- Stop cancels active runtime work and app-supervised child processes while
  preserving app-owned records.
- App-owned SQLite records remain canonical.
- The app can truthfully disclose effective model, provider, tool, approval,
  and instruction-source behavior.
- UI-only approvals, terminal scraping, post-execution audit, and
  best-effort observation are rejected.

## Reducible MVP Capabilities

These may be reduced if needed:

- OpenCode config compatibility claims.
- Root `AGENTS.md` display-only claim for OpenCode-backed sessions.
- Nested `AGENTS.md` invisibility claim.
- Automatic use of existing project OpenCode config.
- Running sessions in arbitrary subdirectories without instruction preflight.
- Native OpenCode instruction compatibility beyond disclosure.

These reductions preserve the desktop coding control-center thesis because
the core loop remains: local project, app-owned session, visible tools,
backend approvals, stop behavior, and bounded persisted records.

## Fallback Decision Order

### 1. Hardened OpenCode Adapter With Mandatory Disclosure

Use this when:

- structured events, pre-execution interception, denial, stop behavior, and
  app-owned record mapping pass;
- the failed behavior is limited to native instruction/config loading;
- the app can enumerate and disclose instruction sources before session start;
- acceptance language is updated so `AGENTS.md` is not described as
  display-only for OpenCode-backed sessions.

Required controls:

- Launch OpenCode directly with isolated XDG dirs, `--pure`, and app-owned
  `OPENCODE_CONFIG_CONTENT`.
- Before session start, scan from project root to session cwd for `AGENTS.md`.
- Read effective `/config` and disclose config-provided `instructions`,
  agent prompts, and instruction-bearing fields.
- Store the disclosed instruction-source inventory in the app-owned session
  context-source summary.
- Block session start if instruction-bearing sources cannot be enumerated,
  classified, or disclosed.
- Keep OpenCode runtime IDs, logs, and persistence as adapter references only.

Current decision:

This is the selected fallback for MVP planning unless a later OpenRouter or
shell policy gate fails.

### 2. App-Controlled Shadow Workspace

Use this when:

- native project config or instruction loading cannot be safely disclosed;
- the product still wants OpenCode behavior but needs stricter instruction
  boundaries.

Required controls:

- Run OpenCode against an app-controlled shadow directory.
- Copy or project only approved files into that runtime view.
- Exclude `opencode.json`, nested instruction files, and other ambient runtime
  config unless explicitly accepted.
- Map approved file changes back to the real project through the backend
  Approval Gateway.

Reject this if:

- filesystem projection or patch mapping becomes more complex than a runtime
  replacement for MVP;
- it weakens Git/file-write correctness.

### 3. Tool Proxy Or Runtime Wrapper

Use this when:

- OpenCode proposes actions but cannot provide enough policy authority for a
  specific tool class.

Required controls:

- The proxy must be pre-execution, not post-execution audit.
- It must preserve structured denial and app-owned ledger records.
- It must not depend on terminal scraping.

Current status:

Not needed for the current evidence because pre-execution interception passed
for the tested protected actions.

### 4. OpenCode Fork

Use this when:

- a narrow runtime patch would disable or expose instruction/config loading;
- upstream cannot provide the needed hook in time;
- the fork surface is smaller than building a replacement runtime.

Reject this if:

- the patch requires broad ownership of provider routing, session storage,
  tools, or model context construction.

### 5. Alternate Runtime Or Direct Provider Mini-Runtime

Use this when:

- OpenCode cannot preserve backend approval authority;
- provider/model routing cannot be constrained or verified;
- instruction loading cannot be disabled, observed, or truthfully disclosed;
- stop behavior cannot terminate app-supervised child processes.

For MVP, a direct provider mini-runtime may reduce scope to assistant text,
explicit file reads, and app-owned shell/file tools, but it must not pretend to
be OpenCode-compatible.

### 6. MVP Scope Reduction

Use this only when:

- all runtime options are too risky for MVP;
- the remaining product still validates local coding trust, persistence, and
  controllability.

Rejected reductions:

- removing backend approval authority;
- allowing protected actions before approval;
- hiding model/provider/instruction behavior;
- relying on UI-only approval or post-execution audit.

## Selected MVP Runtime Path

Select **Hardened OpenCode Adapter With Mandatory Disclosure**.

This accepts that direct OpenCode is not strictly config-isolated. Instead,
the MVP must be honest that OpenCode-backed sessions include runtime-native
instruction sources. The product must disclose those sources before session
start and store them in app-owned records.

This decision resolves FINDING-014 and bounds FINDING-001: the blocker is no
longer an unknown direct-runtime risk, but an accepted fallback path with
explicit implementation requirements.

Later validation resolved the remaining pre-implementation gates. Implementation
planning may proceed through the hardened adapter path; app-build validation,
code-level adapter tests, shell-policy tests, and product QA remain downstream
implementation or release gates.

## Required Follow-Up Edits Before Freeze

- Update ADR-003 to say the MVP runtime path is a hardened OpenCode adapter,
  not unconstrained direct OpenCode.
- Update ADR-013 and acceptance text so root `AGENTS.md` is display-only only
  for app-owned display behavior, not for OpenCode runtime context.
- Add implementation tasks for instruction preflight scanning, `/config`
  capture, disclosure UI, and session context-source persistence.
- Keep OpenCode config compatibility excluded from MVP.
