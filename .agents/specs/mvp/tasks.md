# MVP Proposed Tasks

Status: frozen-for-implementation
Updated: 2026-06-22

These tasks are accepted implementation tasks for the frozen MVP contract. They
are not active work until converted into `.agents/development/progress/` items.

## Sequencing Rules

- Build frontend first from the r04 wireframes before real backend activation.
- Treat `wireframes/r04-single-page-app/` as the functional baseline for
  TASK-001, not as loose visual inspiration. Port r04 screen structure and
  working interactions first; only make production cleanup after parity is
  verified.
- Do not invent UI controls, secondary panels, metadata cards, filters,
  settings abstractions, or route structures during frontend foundation work.
  Any control or layout not present in r04 or explicitly required by the MVP
  contract must be proposed as a later change, not silently added.
- Use `tests/server/` for the mock server harness because this repo's
  implementation contract uses that path, not `test/server/`.
- Every mock-backed phase must state exactly what is mocked.
- Verification must exercise rendered behavior, not only source text. Route
  existence, labels, and `data-*` markers are insufficient evidence by
  themselves.
- Pause for user acceptance at the mock-server review point, first user-flow
  activation, and feature-complete point.
- Security, approval, and policy hardening happen after the product is feature
  complete enough to exercise end-to-end behavior.

## Tasks

### TASK-001: Build Full r04 Frontend

Depends on: review/freeze

Build the entire frontend in `frontend/` from the r04 wireframes and
`wireframes/ui-handoff-spec.md`. This task should establish the actual product
shell frontend by first preserving r04 functional parity, not by redesigning
the UI from scratch.

Parity requirements:

- Preserve every r04 route and the screen-specific structure for App Start, New
  Session, Chat Session, provider/model popovers, Files explorer/editor,
  Terminal, Providers, Add Provider, Models, Runtimes, Configuration, Plugins,
  Skills, and MCP Servers.
- Preserve working r04 interactions before styling or refactoring: left/right
  panel collapse, left/right panel resize, Terminal bottom-panel resize,
  composer attachment preview, approval and branch popovers, provider/model
  popovers, message show/hide, plugin marketplace and connection dialogs, skill
  detail dialog, MCP add-server dialog, and MCP transport switching.
- Preserve the r04 settings screen shapes. Do not collapse settings into a
  generic card/list abstraction unless a later accepted task explicitly changes
  the product UI.
- Do not add new controls or extra surfaces that were not in r04 or the frozen
  MVP contract. In particular, do not add invented right-panel record cards,
  file mutation buttons, extension metadata grids, search/filter controls, or
  settings buttons as a substitute for r04 parity.
- Keep sample data clearly isolated as fixtures or prototype state. Do not
  promote r04 names, URLs, terminal output, provider/model examples, plugin
  records, or copy into product truth.

Use `frontend-design` for design planning and critique passes, then use
`chrisai-coding` over the existing frontend code for maintainability-audit,
logic-review, and HTML/CSS/JavaScript style-guide passes. The maintainability
audit is review-only first and must not justify changing r04 behavior or screen
structure. Apply approved restructuring through the narrowest coding workflow.
Do several internal passes before moving on.

Required TASK-001 verification:

- Render every r04 route in a browser and compare it against the r04 artifact
  for structure, control placement, and screen-specific settings layout.
- Use browser automation to click every collapse control, open every popover or
  dialog listed above, toggle message/detail states, and drag each resize
  handle far enough to prove dimensions change.
- Add tests that fail when collapse, resize, popovers, dialogs, and settings
  route structures are absent. Do not rely on source-string checks as the only
  frontend acceptance evidence.
- Record screenshot or browser-test evidence for all routes before marking the
  item `review`.

Do not pause for user review at this step. Continue until the frontend code is
coherent, maintainable, r04-parity verified, and ready to connect to mocked
server behavior.

### TASK-002: Connect Frontend To Mock Server Harness

Depends on: TASK-001

Create mock backend connections in `tests/server/` that connect to the
frontend. The mock server should perform fake processing that is realistic
enough to exercise frontend loading, success, failure, waiting, and state
transition behavior, then return dummy data.

The mock server must connect to the TASK-001 r04-parity frontend without
changing the accepted screen structure. If a mock endpoint requires a new
frontend control or visible state, document the proposed change and keep it out
of TASK-002 unless it is already part of r04 or the frozen MVP contract.

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

Backend mock parity must preserve the TASK-001/TASK-002 frontend behavior. Do
not replace r04-parity screens with backend-driven simplifications or invented
administrative UI.

This phase must state exactly what is still mocked and must not promote mock
behavior into product-complete evidence.

### TASK-004: Activate First User Flow

Depends on: TASK-003

Implement the backend features needed to complete the first real user journey
and connect them to the frontend. The first journey should be the narrowest
end-to-end path that proves a real user can start from the r04 App Start flow
and reach a useful session state without relying on mock-only behavior.

The first real journey must travel through existing accepted screens and
interactions. Do not introduce alternate onboarding, alternate chat structure,
or simplified settings flows as part of this activation.

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

Each slice must replace mock behavior behind the accepted UI instead of
redesigning the screen. If implementation pressure reveals that r04 needs a UI
change, stop and record a review item before changing the contract.

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
