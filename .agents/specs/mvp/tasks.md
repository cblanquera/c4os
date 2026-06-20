# MVP Proposed Tasks

## TASK-001: Review Imported MVP Scope

Status: proposed
Confidence: imported
MVP: yes
Phase: mvp
Source: `.agents/specs/mvp/brief.md`
Related:
- RISK-003

Run a risk-first review to narrow which capabilities are required for the first customer-usable MVP and which become explicit post-MVP promotion gates.

## TASK-002: Validate Runtime Adapter Target

Status: proposed
Confidence: imported
MVP: yes
Phase: poc
Source: `.agents/specs/mvp/questions.md`
Related:
- ASM-001
- Q-001
- RISK-001

Prove whether OpenCode Runtime, Pi Runtime, or a thin adapter can satisfy session control, streaming, tool, permission, and configuration needs.

## TASK-003: Validate Browser Isolation

Status: proposed
Confidence: imported
MVP: unknown
Phase: poc
Source: `.agents/specs/mvp/questions.md`
Related:
- Q-002
- DEC-004
- AC-007

Prove the native or preview browser surface cannot access privileged app APIs or secrets before promoting it into MVP scope.

## TASK-004: Validate Terminal Lifecycle

Status: proposed
Confidence: imported
MVP: unknown
Phase: poc
Source: `.agents/specs/mvp/questions.md`
Related:
- Q-003
- AC-008

Prove backend-owned terminal lifecycle, trusted cwd validation, sanitized environment, renderer transport, audit records, and cleanup behavior.

## TASK-005: Reconcile Wireframes Into Acceptance

Status: proposed
Confidence: imported
MVP: yes
Phase: mvp
Source: `wireframes/screens.md`
Related:
- AC-009

Convert accepted wireframe behavior into implementation-ready requirements and acceptance criteria before freeze.
