# Unresolved Decisions

This document summarizes ADR decisions that remain open and must be resolved before detailed implementation planning.

## ADR-003: OpenCode Runtime Viability

Summary: OpenCode is the preferred runtime target, but viability is not proven.

Status: Unresolved.

Rationale: MVP depends on structured events, session control, reliable pre-execution tool interception, and visible instruction behavior. If OpenCode cannot provide pre-execution interception for MVP file writes, shell commands, and Git state changes, it is not viable as the direct MVP runtime. If OpenCode invisibly loads root or nested instruction files and the app cannot observe, disclose, or disable that behavior, it is also not viable as the direct MVP runtime. UI-only approvals, post-execution audit logs, terminal scraping, best-effort observation, and invisible instruction injection do not satisfy the accepted MVP gates.

Affected documents:

 - `plans/adr/003-agent-runtime-strategy.md`
 - `plans/specs/architecture-specification.md`
 - `plans/reviews/architecture-review.md`
 - `plans/spikes/SPIKE-001-opencode-runtime-control.md`

Next action: Complete the OpenCode runtime control spike and produce a runtime-control matrix. If the gate fails, choose a wrapper, proxy, fork, runtime replacement, or MVP scope reduction before Phase 1.

## ADR-005: Post-MVP Standards Conformance Levels

Summary: MVP compatibility claims are finalized narrowly, but broader standards conformance levels are not defined.

Status: Unresolved.

Rationale: MVP may claim only OpenRouter-backed model access, local Git project support, root `AGENTS.md` display, and app-owned text-like artifact records. OpenCode runtime settings are app-owned adapter plumbing only and do not imply config compatibility. Broader support for AGENTS.md, SKILL.md, MCP, Codex plugins, OpenCode config, import/export, and round-trip behavior can mean display, parse, execute, import, export, or round-trip, and needs an explicit matrix.

Affected documents:

 - `plans/adr/005-standards-first-interoperability.md`
 - `plans/research/industry-standards-research.md`
 - `plans/spikes/SPIKE-006-standards-conformance-matrix.md`

Next action: Create the standards conformance matrix before making broader compatibility claims.

## ADR-010: Remaining Shell Safety Details

Summary: Project-root file boundaries, secret-deny behavior, symlink boundary behavior, current-user shell execution, normal network access, filtered shell environment behavior, destructive-command approval behavior, manual recovery posture, and bounded shell output persistence are finalized for MVP, but shell output redaction limits remain unresolved.

Status: Unresolved.

Rationale: Shell execution remains high-risk when commands run as the current OS user and shell output redaction is weak.

Affected documents:

 - `plans/adr/010-filesystem-and-shell-access-defaults.md`
 - `plans/specs/security-specification.md`
 - `plans/spikes/SPIKE-003-local-execution-sandboxing.md`

Next action: Complete the local execution sandboxing spike and define detailed redaction matchers and truncation limits for shell output summaries.

## ADR-014: Plugin Format Compatibility

Summary: Codex-compatible plugin layout is the preferred compatibility target, but stability and conformance are unresolved.

Status: Unresolved.

Rationale: Codex plugin fields may drift or be implementation-specific. Plugin execution is excluded from MVP, but future compatibility still needs a clear matrix.

Affected documents:

 - `plans/adr/014-plugin-system-compatibility.md`
 - `plans/specs/plugin-system-specification.md`
 - `plans/spikes/SPIKE-005-plugin-format-and-trust.md`

Next action: Define plugin parse/display/execute conformance and identify stable fields.
