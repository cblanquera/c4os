# Constraints

## CON-001: Backend Safety Boundary Remains Authoritative

Status: proposed
Confidence: evidence-backed
MVP: no
Source: `CONTEXT.md`
Related:
- REQ-003
- RISK-001

### Statement

Post-V1 roadmap work must not move policy authority for files, shell, Git,
secrets, approvals, persistence, or runtime control into the frontend.

## CON-002: Compatibility Claims Need Evidence

Status: proposed
Confidence: evidence-backed
MVP: no
Source: `.agents/specs/c4os-mvp/records/decisions.md`
Related:
- DEC-001
- RISK-003

### Statement

C4OS must not advertise full ecosystem compatibility until conformance,
interoperability, and unsupported-feature behavior are documented and tested.

## CON-003: Scope Acceptance Before Progress

Status: proposed
Confidence: evidence-backed
MVP: no
Source: `.agents/AGENTS.md`, `.agents/specs/c4os-mvp/status.md`
Related:
- REQ-004

### Statement

This roadmap creates proposed planning records only. Active implementation
packets belong in `.agents/progress/` after readiness review and explicit
scope acceptance.
