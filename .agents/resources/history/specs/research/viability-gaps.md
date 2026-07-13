# Research Viability Gaps

Status: frozen-for-mvp-specification
Updated: 2026-06-21

## Gaps

1. MVP scope is intentionally broad: all documented/r04 features are MVP unless explicitly moved out later.
2. Browser product direction now exceeds the earlier isolation POC recommendation and requires implementation proof for a user-owned desktop Browser with project-scoped profile, local file browsing, public web browsing, and request-scoped agent browsing.
3. Terminal product direction is defined as separate user and agent-owned command terminals with deterministic allowlist and audit records.
4. Concurrent sessions/runs are MVP across trusted project folders, with one main run per chat session.
5. Extension install/connect and enabled extension execution are MVP; implementation must preserve provenance, scopes, enablement, and action audit.
6. Checkpoint phases are implementation milestones only and must not be mistaken for MVP scope.

## Recommended Path

Create `.agents/specs/mvp/` from the frozen research records. The MVP spec must define the customer-usable workflow, implementation paths, acceptance criteria, viability gaps, and proposed tasks before any progress item starts.
