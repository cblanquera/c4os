# Unresolved Decisions

This document summarizes ADR decisions that remain open and must be resolved before detailed implementation planning.

## ADR-002: Tauri Versus Electron For Desktop Shell

Summary: Tauri is preferred, but the desktop shell decision remains dependent on runtime integration and browser/artifact requirements.

Status: Unresolved.

Rationale: Tauri fits local-first Rust-backed tooling, but future browser and artifact needs may favor Electron or a Chromium strategy.

Affected documents:

 - `plans/adr/002-desktop-application-shell.md`
 - `plans/specs/architecture-specification.md`
 - `plans/spikes/SPIKE-007-tauri-electron-browser-artifacts.md`

Next action: Complete the Tauri/Electron/browser-artifact spike and confirm launch platform requirements.

## ADR-003: OpenCode Runtime Viability

Summary: OpenCode is the preferred runtime target, but viability is not proven.

Status: Unresolved.

Rationale: MVP depends on structured events, session control, and pre-execution tool interception. If OpenCode cannot provide these, the architecture must change.

Affected documents:

 - `plans/adr/003-agent-runtime-strategy.md`
 - `plans/specs/architecture-specification.md`
 - `plans/reviews/architecture-review.md`
 - `plans/spikes/SPIKE-001-opencode-runtime-control.md`

Next action: Complete the OpenCode runtime control spike and produce a runtime-control matrix.

## ADR-004: Policy Enforcement Authority

Summary: The authoritative policy enforcement layer is not chosen.

Status: Unresolved.

Rationale: Current documents split responsibility between the Rust backend and OpenCode. That creates bypass and audit risks.

Affected documents:

 - `plans/adr/004-policy-enforcement-authority.md`
 - `plans/specs/security-specification.md`
 - `plans/reviews/architecture-review.md`
 - `plans/spikes/SPIKE-002-policy-enforcement-authority.md`

Next action: Decide whether the backend, runtime, or a mandatory tool proxy is authoritative.

## ADR-005: Standards Conformance Levels

Summary: Standards-first remains a principle, but exact conformance levels are not defined.

Status: Unresolved.

Rationale: Support for AGENTS.md, SKILL.md, MCP, Codex plugins, OpenCode config, and OpenRouter can mean display, parse, execute, import, export, or round-trip.

Affected documents:

 - `plans/adr/005-standards-first-interoperability.md`
 - `plans/research/industry-standards-research.md`
 - `plans/spikes/SPIKE-006-standards-conformance-matrix.md`

Next action: Create the standards conformance matrix.

## ADR-010: Local Execution Sandbox And Network Policy

Summary: Project-root scoping and shell approvals are selected for MVP, but sandboxing and network behavior remain unresolved.

Status: Unresolved.

Rationale: Shell execution is high-risk if commands run as the user with broad network and filesystem access.

Affected documents:

 - `plans/adr/010-filesystem-and-shell-access-defaults.md`
 - `plans/specs/security-specification.md`
 - `plans/spikes/SPIKE-003-local-execution-sandboxing.md`

Next action: Complete the local execution sandboxing spike and define deny patterns, symlink behavior, network defaults, and destructive command categories.

## ADR-014: Plugin Format Compatibility

Summary: Codex-compatible plugin layout is the preferred compatibility target, but stability and conformance are unresolved.

Status: Unresolved.

Rationale: Codex plugin fields may drift or be implementation-specific. Plugin execution is excluded from MVP, but future compatibility still needs a clear matrix.

Affected documents:

 - `plans/adr/014-plugin-system-compatibility.md`
 - `plans/specs/plugin-system-specification.md`
 - `plans/spikes/SPIKE-005-plugin-format-and-trust.md`

Next action: Define plugin parse/display/execute conformance and identify stable fields.

## ADR-023: Telemetry And Privacy

Summary: Product telemetry, diagnostics, and data movement policy remain undefined.

Status: Unresolved.

Rationale: Local-first claims are incomplete while model-provider routing, logs, support bundles, MCP, plugins, and telemetry data movement are undefined.

Affected documents:

 - `plans/adr/00-adr-candidate-list.md`
 - `plans/reviews/adr-architecture-review-report.md`
 - `plans/spikes/SPIKE-009-provider-privacy-cost.md`
 - `plans/spikes/SPIKE-014-memory-session-retention.md`

Next action: Define what data leaves the machine and whether telemetry is none, local diagnostics only, opt-in, or enterprise-controlled.

