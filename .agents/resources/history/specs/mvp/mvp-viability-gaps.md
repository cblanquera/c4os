# MVP Viability Gaps

Status: frozen-for-implementation
Updated: 2026-06-21

## Gaps To Resolve During Implementation

### GAP-001: Broad MVP Scope

The MVP intentionally includes the full documented/r04 product slice. Planning
must avoid silently shrinking Browser, Terminal, extension, concurrency, or
resume scope through checkpoint language.

Resolution path: freeze this spec as the full contract, then use checkpoints
only as progress gates.

### GAP-002: Browser Implementation Boundary

Browser must support project-scoped profiles, local file browsing, public
browsing, request-scoped agent browsing, logged-in use when clearly requested,
agent action records, privileged API isolation, and no downloads.

Resolution path: implement Browser behind app-owned authority and audit records;
verify with product acceptance, not only isolation POCs.

### GAP-003: Terminal Policy Boundary

Terminal must support user and agent-owned command surfaces, backend lifecycle,
deterministic allowlist, approvals, sanitized execution, cancellation, cleanup,
and audit records.

Resolution path: build backend-owned terminal state first, then layer renderer
transport and policy UX on top.

### GAP-004: Extension Runtime Impact

Plugins, skills, and MCP servers are MVP but can affect runtime only after
explicit enablement. MCP invocation must remain explicit.

Resolution path: model extension records, enablement gates, prompt tags, and
audit records before integrating runtime effects.

### GAP-005: Concurrency And Resume

Concurrent runs across trusted folders can cross approvals, outputs, cwd,
artifacts, runtime events, cancellation, Browser state, terminal state, and
extension state.

Resolution path: define app-owned records and isolation keys before adding
multi-run UI polish.

### GAP-006: Product State Versus Descriptor State

Workspace descriptors must not become the dumping ground for secrets,
transcripts, artifact archives, or private runtime state.

Resolution path: keep `.c4os` descriptors reference-only and store operational
state in app-owned local storage.

### GAP-007: Real Behavior Versus Wireframe Simulation

r04 shows interaction intent with sample data. MVP viability requires real
state, real persistence, real policy checks, and real backend-owned boundaries.

Resolution path: use r04 as behavioral acceptance input, not implementation
evidence.
