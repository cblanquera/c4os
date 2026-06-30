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
- TASK-001 must produce a production-quality frontend foundation, not a copied
  wireframe. r04 parity is the minimum floor; the completed frontend should
  preserve r04 behavior and information architecture while improving the visual
  presentation enough to be a credible desktop product shell.
- Do not invent UI controls, secondary panels, metadata cards, filters,
  settings abstractions, or route structures during frontend foundation work.
  Any control or layout not present in r04 or explicitly required by the MVP
  contract must be proposed as a later change, not silently added.
- After TASK-001 passes review, later tasks may replace static fixtures,
  placeholder data, and static component behavior with dynamic, backend-backed,
  or working equivalents behind the accepted UI. Later tasks must not add new
  visible UI components, controls, panels, menus, cards, settings abstractions,
  or route structures unless a change is explicitly accepted and documented
  before implementation.
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
shell frontend by preserving r04 functional parity and then applying a
production visual treatment. The result should be a stable product shell ready
for mocked server state in TASK-002, not a throwaway prototype and not a
copy/paste of the grayscale wireframe.

Purpose:

- Convert the accepted r04 product shape into real app frontend code.
- Preserve the hard-won r04 behavior, screen inventory, information
  architecture, and control placement so later tasks can connect real state
  without rediscovering the UI contract.
- Improve presentation, hierarchy, typography, spacing, color, controls,
  panels, and settings polish enough that the first reviewable frontend feels
  like C4OS product software instead of a planning artifact.
- Leave a tested shell that TASK-002 can connect to mock server data without
  rewriting routes, screens, or interactions.

Allowed TASK-001 implementation scope:

- `frontend/` for the product shell frontend.
- Frontend behavior tests using filenames such as `tests/frontend-*.test.*`
  when needed for rendered browser verification.
- Frontend-only support harness files under `tests/support/` when needed to run
  TASK-001 verification locally.
- Minimal package script updates when needed to expose TASK-001 frontend dev,
  test, or acceptance commands.

TASK-001 must not edit `backend/` or `tests/server/`. If frontend work exposes
a backend or mock-server concern, record it for the next task instead of adding
server behavior during TASK-001.

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

Production presentation requirements:

- Do not keep the r04 white/grayscale wireframe treatment as the final TASK-001
  visual result. Apply a restrained production desktop-app visual system while
  preserving r04 structure and behavior.
- Improve visual hierarchy, typography, spacing, color tokens, panel surfaces,
  button/control treatment, settings readability, and Browser/Files/Terminal
  polish without adding, removing, renaming, or relocating product controls.
- Keep the interface utilitarian and work-focused. Do not turn it into a
  marketing page, decorative dashboard, or generic chat clone.
- If a production visual improvement appears to require a structural change,
  record it for review and leave it out of TASK-001.

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
- Keep new tests and harnesses limited to frontend rendering, interaction,
  screenshot, and TASK-001 acceptance verification. Do not create the TASK-002
  mock server harness or any `tests/server/` content.
- Record screenshot or browser-test evidence for all routes before marking the
  item `review`. Screenshot review must verify both r04 structural parity and
  production visual treatment.

Do not pause for user review at this step. Continue until the frontend code is
coherent, maintainable, r04-parity verified, visually production-treated, and
ready to connect to mocked server behavior.

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

TASK-003 must also document and expose the native desktop app menu contract at
the OS level. The menu is part of the desktop shell, not an in-app replacement
for accepted r04 controls. Required menu items:

- `File > Open Workspace`
- `File > Save Workspace`
- `File > Save File`; enabled only when the file editor surface is open and a
  file can be saved
- `Edit`; enabled when focus is in an editable context such as the chat prompt,
  Browser address bar, file editor, provider/settings input fields, or other
  text-entry controls
- `Edit > Undo`
- `Edit > Redo`
- `Edit > Select All`
- `Edit > Cut`
- `Edit > Copy`
- `Edit > Paste`

TASK-003 may wire these menu items to backend/mock handlers and focus state,
but must not add duplicate in-app toolbar buttons or visible UI components for
these commands unless a later accepted change explicitly revises the UI
contract.

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

### TASK-005: OpenRouter Chat Session Review Slice

Depends on: TASK-004 acceptance

Activate the real chat-session experience behind the accepted r04 shell using a
real OpenRouter API key and a reasoning-capable model. This is the first
feature-unlock review slice after TASK-004.

The goal is to review the user-visible chat session behavior before broader
feature unlocking continues. A user should be able to open a trusted folder,
start a chat, use a configured OpenRouter provider/model, see live thinking or
reasoning output when the selected model returns it, see structured activity
for what the app/runtime is doing, and receive a final assistant response.

Required behavior:

- Use a real OpenRouter API key through a backend-owned credential reference or
  review-only secure local configuration. Do not write raw keys to workspace
  descriptors, transcripts, normal app state, logs, or frontend fixtures.
- Use the OpenRouter chat completions API with streaming enabled for the live
  chat path.
- Send reasoning configuration for reasoning-capable models and preserve
  compatibility with models that do not return reasoning tokens.
- Render streamed reasoning/thinking chunks as distinct structured activity
  before the final response when OpenRouter returns `reasoning_details` or
  equivalent reasoning text.
- Render app/runtime activity separately from assistant content, so the user can
  tell what C4OS is doing versus what the model is saying.
- Persist C4OS-owned workspace, session, message, run, and runtime-event
  records needed to keep the session list and active thread coherent during the
  review.
- Keep the accepted r04 chat/session layout. Do not add alternate onboarding,
  alternate chat structure, new settings abstractions, or new route structures.
- Pause for user review after this real OpenRouter chat slice.

Still allowed to remain mocked in this slice: Files content and mutation,
Browser, Terminal, extensions, local memory, artifacts, broad restart/resume,
full provider-management polish, advanced approval policy, audit-log hardening,
and non-chat feature slices.

### TASK-005A: Scoped Frontend State And DOM Updates

Depends on: TASK-005 acceptance

Consolidate the app-owned frontend state and rerender boundaries before adding
provider/model management. This is a cleanup bridge from the TASK-005 review
feedback, not a new feature slice and not a framework migration.

Purpose:

- Keep chat/session UI state explicit before provider/model, runtime, Files,
  Browser, Terminal, approvals, and persistence work add more moving parts.
- Stop local interactions from replacing the whole shell when only one scoped
  surface changes.
- Preserve the no-build frontend and existing accepted r04 shell while reducing
  visible refresh/fade behavior and future state-management tech debt.

Required behavior:

- Introduce a small app-owned state store or equivalent state boundary for
  route, workspace, active session, per-session right-panel tool state, turns,
  connector run state, and composer-local state.
- Replace global `render()` calls for local chat/tool interactions with scoped
  DOM updates where practical: right-panel tab switches, work-log disclosure,
  active-turn streaming/progress updates, and composer updates.
- Keep right-panel active tool state scoped per chat/new-session surface.
- Keep transcript turn DOM keyed by stable turn/session identity so appending a
  turn does not replace prior turns.
- Do not add React, Preact, JSX, a bundler, or a new build pipeline in this
  cleanup task. Reconsider a framework only if this scoped-store pass proves
  insufficient.
- Preserve the accepted r04 routes and visible UI structure. Do not add new
  panels, settings abstractions, or alternate chat layouts.

Verification:

- Tests prove right-panel tab changes do not replace the chat transcript DOM.
- Tests prove work-log expansion does not replace message bubbles.
- Tests prove streaming/progress updates affect only the active turn.
- Existing TASK-001 through TASK-005 tests continue to pass.

### TASK-006: Provider And Model Management Slice

Depends on: TASK-005A acceptance

Replace provider/model mock records behind the accepted Settings and composer
surfaces. This slice should support OpenAI-compatible provider profiles,
non-secret provider/model records, secure key status, model discovery or manual
model entry, model enablement, and per-session model selection.

Pause for review if provider/model UX pressure requires changing the accepted
r04 UI contract.

### TASK-007: Runtime Adapter And Persistent Sessions Slice

Depends on: TASK-006 acceptance

Replace remaining mock session/runtime behavior with C4OS-owned session, run,
message, and runtime-reference records behind the OpenCode-first adapter. The
runtime must stay behind the C4OS boundary, and user-facing records must remain
C4OS workspace/session/run records.

### TASK-008: Files Slice

Depends on: TASK-007 acceptance

Replace Files mock behavior with trusted-root browsing, open-file/code view,
editing, saving, reverting, guarded mutation, and dirty-state behavior behind
the accepted right-panel Files surfaces.

### TASK-009: Artifact And Safe HTML Preview Slice

Depends on: TASK-008 acceptance

Replace artifact/preview mocks with product-owned artifact records and safe
rendering for generated or untrusted HTML inside the existing right-panel
Browser/Preview tab. This slice must not add a new Artifacts tab, reuse the
Terminal tab, or claim full Browser or security hardening complete beyond the
artifact preview boundary.

### TASK-010: Browser Slice

Depends on: TASK-009 acceptance

Replace Browser mock behavior with the MVP Browser surface: project-scoped
profile state, local file opening, public browsing, request-scoped agent
browsing when clearly requested, and recorded agent Browser actions. Downloads
remain out of scope.

### TASK-010A: Browser Address Bar And Local Target UI

Depends on: TASK-010 acceptance

Expose the TASK-010 Browser capabilities through the accepted right-panel
Browser surface: make the existing Browser address bar user-editable for public
URLs, resolve user-entered local file targets to canonical absolute `file:///`
URLs, keep agent-entered local Browser targets behind trusted-root authority,
and show bounded success/failure state without adding Browser routes, panels,
tabs, alternate layouts, downloads, React/Preact/JSX, a bundler, or settings
abstractions. This follow-up completes the user-testable Browser navigation
path for the MVP surface without expanding Browser security hardening beyond
the TASK-010/TASK-016 boundary.

### TASK-010B: Native Browser Webview Or External-Open Fallback

Depends on: TASK-010A acceptance

Resolve the iframe-blocked public website limitation in the Browser surface
with the accepted raw Wry native webview-backed Browser host. Public Browser
pages must no longer rely on the DOM iframe path that fails on
`X-Frame-Options`, CSP `frame-ancestors`, or equivalent browser restrictions.
Preserve the existing Browser / Files / Terminal right-panel tab contract,
keep TASK-009 artifact previews on their distinct sandboxed preview path, keep
local-file Browser previews separate from general public browsing, keep user
browsing separate from agent browsing authority, and do not add downloads,
external-open fallback UI, unrelated settings abstractions, or alternate app
layouts.

### TASK-010C: Artifact Preview Type Rendering

Depends on: TASK-010B acceptance

Extend the TASK-009 artifact preview path inside the accepted Browser/Preview
surface so product-owned artifact records render according to MIME type or file
extension. Support safe previews for common artifact types such as generated
or untrusted HTML, Markdown, images, PDF, plain text, JSON, and code where
reasonable for the MVP. Keep generated/untrusted HTML sandboxed, keep artifact
previews distinct from general Browser navigation, do not render artifacts in
Terminal, and do not add a new Artifacts tab, route, panel, alternate layout,
downloads, React/Preact/JSX, a bundler, or settings abstraction.

### TASK-011: Terminal Slice

Depends on: TASK-010C acceptance

Replace Terminal mock behavior with backend-owned user terminal and agent
command terminal surfaces, including lifecycle ownership, cwd validation,
sanitized environment, output streaming, cancellation, failure reporting, and
action records. Approval-policy hardening remains TASK-016.

### TASK-012: Settings IA And Extension Records Slice

Depends on: TASK-011 acceptance

Replace settings, plugin, skill, and MCP mock records with app-owned records
behind the accepted Settings IA. Runtime impact remains disabled until explicit
enablement.

### TASK-013: Concurrency And Restart/Resume Slice

Depends on: TASK-012 acceptance

Implement concurrent session/run isolation across trusted folders and restart/
resume behavior for transcripts, runtime references, provider/model context,
artifacts, Browser/file/terminal/extension action records, and run state needed
for continuation.

### TASK-014: Local Memory, Action Records, And Audit Records Slice

Depends on: TASK-013 acceptance

Implement app-owned local memory records, action records, and audit records
needed by the MVP surfaces. Keep these separate from raw provider state,
workspace descriptors, and runtime-local storage.

### TASK-015: Pause At Feature Complete

Depends on: TASK-014 complete

When all r04 MVP features are unlocked and connected, stop and pause for user
acceptance before security and approval hardening begins. State the completed
feature list, the acceptance evidence, and any mocks or shortcuts still
remaining.

### TASK-016: Implement Security And Approval Policies

Depends on: TASK-015 acceptance

Implement security policies, approval policies, trusted-root containment,
descriptor safety, secure key storage, isolated HTML preview, Browser authority
rules, terminal command policy, extension enablement gates, and audit logs
against the now feature-complete product surface.

TASK-016 must provision a formal tool approval configuration store for the
runtime tool gateway. The storage model must include project/workspace-level
defaults scoped to the trusted project folder, optional per-session overrides
that can narrow or request policy changes for a chat session, and a run-scoped
effective-policy snapshot persisted with tool/action/audit records. The config
shape should map stable tool identities such as `terminal.run`, `files.read`,
`files.write`, `browser.open`, and `artifact.preview` to enabled state, access
level, and approval policy. Workspace/project defaults are the source of truth
for folder-scoped trust; session overrides must not silently widen beyond tool
maximum authority or workspace policy.

This phase should harden real behavior, not merely document expected policy.

### TASK-017: Complete Remaining Integration And Release Readiness

Depends on: TASK-016

Complete any remaining required implementation, integration, migration,
packaging, QA, accessibility, restart/resume, failure-state, and acceptance-test
work needed for a release-ready MVP.

TASK-017 must resolve the deferred runtime tool-output reflection gap: provider
or runtime command execution must arrive as structured C4OS tool lifecycle
events such as `tool_call_requested`, `tool_output_delta`, and
`tool_call_completed`, with `terminal.run` executed through the gateway and
reflected in the Agent terminal. Assistant prose or markdown that merely
contains command output is not a valid Agent terminal source of truth.

Run backend, frontend, integration, and MVP acceptance verification. Do not
claim product completion until all acceptance criteria pass with real behavior
or explicitly accepted remaining mocks.
