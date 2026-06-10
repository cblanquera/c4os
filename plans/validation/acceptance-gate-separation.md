# Acceptance Gate Separation

Date: 2026-06-10

Related finding:

- FINDING-011

Related evidence:

- `plans/validation/evidence-matrix.md`
- `plans/validation/decision-log.md`
- `plans/validation/FINDING-001-pre-execution-interception.md`
- `plans/validation/FINDING-001-stop-behavior.md`
- `plans/validation/FINDING-001-app-owned-record-mapping.md`
- `plans/validation/FINDING-001-runtime-config-isolation.md`
- `plans/validation/FINDING-001-instruction-loading-observability.md`
- `plans/validation/FINDING-004-openrouter-runtime-verification.md`
- `plans/validation/FINDING-003-shell-security-policy.md`
- `plans/validation/FINDING-014-runtime-fallback-decision.md`
- `plans/validation/FINDING-009-015-tauri-platform-matrix.md`

## Purpose

FINDING-011 warned that the planning corpus mixed Phase 0 architecture
viability gates with ordinary product acceptance criteria. This artifact
separates those gate types before freeze-and-plan work begins.

## Gate Classes

### Phase 0 architecture gates

These gates decide whether the MVP architecture is viable before feature
implementation starts:

- Runtime integration surface exists.
- Structured runtime events are observable without terminal scraping.
- File writes, shell commands, and runtime-proposed Git state changes are
  intercepted before execution.
- Denials are returned as structured runtime results.
- Stop behavior aborts the runtime stream and app-supervised child processes.
- App-owned records remain canonical over runtime-native records.
- OpenRouter provider routing, model match, and credential-reference stability
  are verified.
- Shell environment, redaction, truncation, destructive-command, and
  unclassifiable-command policies are concrete.
- Runtime fallback order is explicit.
- MVP platform matrix is named.

Status: passed for implementation planning, with the fallback and scope changes
listed below.

### Accepted fallback or scope-change gates

These gates produced failures or caveats, but they are resolved by explicit
MVP scope decisions instead of being carried as product QA failures:

- Runtime config isolation failed for project-level OpenCode instructions.
  Resolution: do not use unconstrained direct OpenCode. Use the hardened
  OpenCode adapter with mandatory instruction/config disclosure.
- Strict OpenRouter context exclusion failed for OpenCode-backed sessions.
  Resolution: replace strict exclusion language with bounded, disclosed
  context-source categories.
- Root `AGENTS.md` display-only behavior remains true only for app-owned UI
  display. OpenCode-backed sessions must disclose runtime-native instruction
  sources before session start.

Status: accepted for implementation planning.

### Deferred app-build validation gates

These gates cannot be completed until a real app build or prototype exists.
They should block release or milestone acceptance, not implementation start:

- Tauri MVP UI validation on mandatory launch platforms.
- Text-like artifact preview validation in the built shell.
- Code-level tests for shell filtering, redaction, truncation, destructive
  classification, and unclassifiable-command handling.
- Code-level adapter tests for instruction-source inventory, `/config`
  redaction, approval routing, stop mapping, and app-owned record persistence.

Status: deferred to implementation and release validation.

### Product QA acceptance

These checks validate whether the implemented product behaves correctly for
users:

- Project open and root boundary journeys.
- Session creation, transcript, stop, retry, resume, and failure journeys.
- Approval prompts and durable approval ledger behavior.
- Shell execution UX, summaries, redaction display, and denial display.
- File write previews, changed-file lists, and Git diff inspection.
- OpenRouter credential setup and active-session update/revoke blocking.
- `AGENTS.md` display and instruction-source disclosure copy.
- Local persistence, artifact records, and app restart behavior.

Status: not started; this belongs to implementation QA.

## Separation Rules

- Phase 0 gates answer whether the architecture can be implemented.
- Product QA acceptance answers whether the implemented product works for
  users.
- A failed architecture gate may be resolved by an explicit fallback or scope
  change if the new behavior is documented and testable.
- App-build-only checks must not block implementation start. They become
  release or milestone gates.
- Acceptance documents may reference Phase 0 evidence, but they must not treat
  already-resolved architecture decisions as ordinary QA scenarios.

## Decision

FINDING-011 is resolved for implementation planning.

The corpus is ready to enter freeze-and-plan with these boundaries:

- Phase 0 runtime/provider/shell/fallback/platform evidence is recorded.
- Hardened OpenCode adapter is the selected runtime path.
- Implementation must include the deferred app-build and code-level validation
  tests before release readiness can be claimed.
