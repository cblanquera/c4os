# Research Proposed Tasks

## TASK-001: Review Imported MVP Scope

Status: done
Confidence: user-accepted
MVP: yes
Phase: mvp
Source: `.agents/specs/research/brief.md`, `.agents/references/research/grill-session.md`
Related:
- RISK-003

Grill review resolved that all documented/r04 features are MVP unless explicitly moved out later. Checkpoint phases may sequence implementation progress, but they are not the MVP boundary.

## TASK-002: Validate Runtime Adapter Target

Status: done
Confidence: evidence-backed
MVP: yes
Phase: poc
Source: `.agents/specs/research/questions.md`, `.agents/specs/research/poc/brief.md`, `.agents/specs/research/poc/results.md`
Related:
- ASM-001
- Q-001
- RISK-001

OpenCode Runtime and Pi Runtime were compared against the shared adapter contract. The result supports a thin C4OS-owned runtime adapter, with OpenCode viable for server-backed coding runtime control and Pi viable for in-process approval interception.

## TASK-003: Validate Browser Isolation

Status: done
Confidence: user-accepted
MVP: yes
Phase: poc
Source: `.agents/specs/research/questions.md`, `.agents/specs/research/poc/results.md`, `.agents/references/research/grill-session.md`
Related:
- Q-002
- DEC-004
- AC-007

Browser isolation POCs identified implementation risks, but grill review promoted Browser as MVP product scope: user-owned desktop Browser with project-scoped profile, local file browsing, public web browsing, request-scoped agent browsing, and recorded agent Browser actions.

## TASK-004: Validate Terminal Lifecycle

Status: done
Confidence: user-accepted
MVP: yes
Phase: poc
Source: `.agents/specs/research/questions.md`, `.agents/specs/research/poc/results.md`, `.agents/references/research/grill-session.md`
Related:
- Q-003
- AC-008

Backend-owned terminal lifecycle was validated on macOS through Python and Rust PTY proofs. Grill review promoted Terminal as MVP product scope with separate user terminal and agent-owned command terminal, deterministic allowlist, approvals, and audit records.

## TASK-005: Reconcile Wireframes Into Acceptance

Status: done
Confidence: evidence-backed
MVP: yes
Phase: mvp
Source: `wireframes/screens.md`, `wireframes/ui-handoff-spec.md`, `.agents/specs/research/acceptance.md`
Related:
- AC-009
- AC-010
- AC-011
- AC-012
- AC-013
- AC-014
- AC-015
- EVD-009
- DEC-009

Accepted r04 wireframe behavior was promoted into implementation-ready requirements and acceptance criteria. Placeholder data, sample copy, and grayscale styling remain illustrative unless separately promoted.
