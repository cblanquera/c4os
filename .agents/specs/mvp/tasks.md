# MVP Proposed Tasks

Status: frozen-for-implementation
Updated: 2026-06-21

These tasks are accepted implementation tasks for the frozen MVP contract. They
are not active work until converted into `.agents/development/progress/` items.

## Sequencing Rules

- Build frontend first from the r04 wireframes before real backend activation.
- Use `tests/server/` for the mock server harness because this repo's
  implementation contract uses that path, not `test/server/`.
- Every mock-backed phase must state exactly what is mocked.
- Pause for user acceptance at the mock-server review point, first user-flow
  activation, and feature-complete point.
- Security, approval, and policy hardening happen after the product is feature
  complete enough to exercise end-to-end behavior.

## Tasks

### TASK-001: Build Full r04 Frontend

Depends on: review/freeze

Build the entire frontend in `frontend/` from the r04 wireframes and
`wireframes/ui-handoff-spec.md`. This task should finalize the actual frontend
code intended for the product shell, not a throwaway prototype.

Use `frontend-design` for design planning and critique passes, then use
`chrisai-coding` over the existing frontend code for maintainability-audit,
logic-review, and HTML/CSS/JavaScript style-guide passes. The maintainability
audit is review-only first; apply approved restructuring through the narrowest
coding workflow. Do several internal passes before moving on.

Do not pause for user review at this step. Continue until the frontend code is
coherent, maintainable, and ready to connect to mocked server behavior.

### TASK-002: Connect Frontend To Mock Server Harness

Depends on: TASK-001

Create mock backend connections in `tests/server/` that connect to the
frontend. The mock server should perform fake processing that is realistic
enough to exercise frontend loading, success, failure, waiting, and state
transition behavior, then return dummy data.

This phase must clearly state what is mocked, including fake workspace trust,
fake provider/model records, fake session creation, fake agent processing,
fake structured thread and run events, fake files, fake artifacts and previews,
fake Browser state, fake terminal output, fake settings IA, fake extension
records, fake approvals, fake local memory, fake action records, fake workspace
descriptors, and fake persistence.

Pause for user acceptance after this task.

### TASK-003: Build Backend Mock Parity

Depends on: TASK-002 acceptance

Develop the real backend surface in `backend/`, but keep it returning the same
mock data and fake processing behavior proven through `tests/server/`. This
creates backend/frontend integration parity without claiming real provider,
runtime, filesystem, Browser, terminal, extension, security, approval, or
persistence behavior is complete.

This phase must state exactly what is still mocked and must not promote mock
behavior into product-complete evidence.

### TASK-004: Activate First User Flow

Depends on: TASK-003

Implement the backend features needed to complete the first real user journey
and connect them to the frontend. The first journey should be the narrowest
end-to-end path that proves a real user can start from the r04 App Start flow
and reach a useful session state without relying on mock-only behavior.

Pause for user acceptance after this task.

### TASK-005: Unlock Feature Slices One At A Time

Depends on: TASK-004 acceptance

After the first user flow is accepted, unlock one feature slice at a time until
the full documented/r04 MVP is feature complete. Each slice should replace mock
behavior with real behavior, connect the result to the frontend, verify it, and
state any remaining mocks before starting the next slice.

Expected slices include workspace/trust records, workspace descriptors,
provider/model setup, runtime adapter/session behavior, structured thread/run
events, Files, safe artifact/HTML preview, Browser, Terminal, settings IA,
extensions, concurrency, restart/resume, local memory, action records, and audit
records. The exact order can be adjusted during progress planning, but scope
must not shrink.

### TASK-006: Pause At Feature Complete

Depends on: TASK-005 complete

When all r04 MVP features are unlocked and connected, stop and pause for user
acceptance before security and approval hardening begins. State the completed
feature list, the acceptance evidence, and any mocks or shortcuts still
remaining.

### TASK-007: Implement Security And Approval Policies

Depends on: TASK-006 acceptance

Implement security policies, approval policies, trusted-root containment,
descriptor safety, secure key storage, isolated HTML preview, Browser authority
rules, terminal command policy, extension enablement gates, and audit logs
against the now feature-complete product surface.

This phase should harden real behavior, not merely document expected policy.

### TASK-008: Complete Remaining Integration And Release Readiness

Depends on: TASK-007

Complete any remaining required implementation, integration, migration,
packaging, QA, accessibility, restart/resume, failure-state, and acceptance-test
work needed for a release-ready MVP.

Run backend, frontend, integration, and MVP acceptance verification. Do not
claim product completion until all acceptance criteria pass with real behavior
or explicitly accepted remaining mocks.
