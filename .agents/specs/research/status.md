# Research Status

Status: frozen-for-implementation-planning
Updated: 2026-06-20

## Classification

- Setup mode: import
- Source confidence: imported
- Implementation state: not started in this repo
- Freeze state: frozen for implementation planning

## Readiness

The imported plans, POCs, r04 wireframes, and grill answers are frozen for implementation planning. The MVP is intentionally the full documented/r04 product scope, not a smaller checkpoint slice. Checkpoint phases may be used for implementation progress, but the product is not MVP until the documented features are complete.

## Frozen Decisions

- OpenCode is selected as the first backend behind a thin C4OS-owned adapter; Pi remains a later adapter target.
- Browser is MVP as a user-owned desktop Browser with project-scoped profile, public web access, local file browsing, request-scoped agent browsing, and recorded agent Browser actions.
- Terminal is MVP as separate user terminal and agent-owned command terminal with deterministic allowlist, approvals, and audit records.
- Concurrent sessions/runs across trusted project folders are MVP, with one main run per chat session.
- Extension install/connect for plugins, skills, and MCP servers is MVP, with explicit per-extension enablement before runtime impact.
- r04 wireframe behavior is MVP behavioral handoff; final visual design and product copy remain separate unless explicitly documented.
- Freeze artifacts now exist in `mvp-freeze.md`, `implementation-checkpoint-plan.md`, and `docs/adr/`.
