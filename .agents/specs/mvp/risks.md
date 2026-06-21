# MVP Risks

Status: frozen-for-implementation
Updated: 2026-06-21

## Risks

### RISK-001: Runtime Coupling

Status: mitigated by design
Source: `.agents/specs/research/risks.md`, `.agents/specs/research/poc/results.md`

OpenCode concepts could leak into the product model. The MVP mitigates this by
requiring a thin C4OS-owned adapter and C4OS-owned records for sessions, runs,
approvals, artifacts, providers, extensions, and persistence.

### RISK-002: Security Boundary Drift

Status: open for implementation
Source: `.agents/specs/research/risks.md`, `.agents/context/technical-specs.md`

Browser, terminal, file editing, artifact preview, and extensions expand local
authority. Implementation must preserve trusted roots, approval policy,
extension enablement, sanitized terminal execution, isolated previews, and audit
records.

### RISK-003: Browser Scope Complexity

Status: open for implementation
Source: `docs/adr/0001-browser-and-agent-authority.md`, `.agents/specs/research/viability-gaps.md`

The Browser MVP exceeds a constrained preview-only surface. The implementation
must reconcile project-scoped profiles, local file browsing, public browsing,
request-scoped agent browsing, logged-in use when requested, no downloads, and
privileged API isolation.

### RISK-004: Terminal Command Policy Errors

Status: open for implementation
Source: `.agents/specs/research/decisions.md`, `.agents/specs/research/poc/results.md`

Command classification mistakes could allow mutation without approval or block
expected read-only work. The allowlist must be deterministic, exact-patterned,
audited, and overrideable by workspace config.

### RISK-005: Extension Runtime Impact

Status: open for implementation
Source: `docs/adr/0002-extension-enablements-and-prompt-tags.md`

Plugins, skills, and MCP servers may affect tools, prompts, state, and runtime
behavior. MVP records must show provenance, scopes, data sharing, runtime/tool
access, enabled state, invocation path, and action audit.

### RISK-006: Provider Complexity

Status: open
Source: `.agents/specs/research/risks.md`

Provider-native APIs could consume MVP scope. MVP starts with
OpenAI-compatible provider profiles and secure BYOK storage.

### RISK-007: Persistence Confusion

Status: open
Source: `.agents/specs/research/risks.md`

Users may expect `.c4os` descriptors to carry all state. MVP must distinguish
non-secret workspace descriptors from app-owned local state for transcripts,
artifacts, approvals, audit logs, and runtime references.

### RISK-008: Checkpoint Scope Drift

Status: mitigated by spec wording
Source: `.agents/specs/research/implementation-checkpoint-plan.md`

Implementation checkpoints could be mistaken for smaller MVP boundaries. This
spec treats checkpoints as sequencing gates only.

### RISK-009: UI Simulation Mistaken For Product

Status: mitigated by acceptance
Source: `wireframes/ui-handoff-spec.md`

r04 prototype behavior is a handoff, not product code. Acceptance requires real
state, product records, and backend-owned boundaries rather than hardcoded
sample data or simulated completion.
