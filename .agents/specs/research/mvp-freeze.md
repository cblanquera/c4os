# MVP Freeze

Status: frozen-for-implementation-planning
Updated: 2026-06-21
Source:
- `.agents/references/research/grill-session.md`
- `.agents/specs/research/requirements.md`
- `.agents/specs/research/acceptance.md`
- `wireframes/ui-handoff-spec.md`

## Freeze Rule

The MVP is the full documented/r04 product scope unless explicitly moved out in
a later accepted decision. Checkpoints are implementation sequencing tools, not
MVP boundaries.

## MVP Scope

- Folder-first trusted workspace entry.
- Provider setup with OpenAI-compatible provider profiles and secure key
  storage.
- OpenCode first behind a thin C4OS-owned runtime adapter; Pi remains a later
  adapter target.
- Persistent sessions with visible agent status, response history, runtime
  state, artifacts, approvals, and resumability after app restart.
- Concurrent sessions/runs across trusted project folders, with one main run
  per chat session.
- Agent authority scoped by active project and user request: project file
  reads/writes are allowed when implied by the request; outside-project file
  access requires explicit permission.
- Browser as a user-owned desktop surface with project-scoped profile, local
  file browsing, public web browsing, request-scoped agent browsing, logged-in
  use when clearly requested, and recorded agent Browser actions. Downloads are
  not MVP.
- Files explorer/editor for trusted project file inspection and editing.
- Separate user terminal and agent-owned command terminal, both backend-owned.
- Agent-owned terminal read-only allowlist using exact command patterns from
  app-bundled defaults, overrideable by workspace config.
- Extension install/connect for plugins, skills, and MCP servers, with
  explicit per-extension enablement before runtime impact.
- Chat prompt extension tags: `$skill`, `@plugin`, and `^mcp`; MCP invocation
  is explicit only.
- r04 shell behavior: App Start, New Session, Chat Session, provider/model
  selectors, Browser, Files, Terminal, and Settings surfaces.

## Acceptance Bar

MVP is not complete until the documented/r04 feature set works as an integrated
product. Minimum product acceptance includes:

- trusted folder opened
- provider configured
- OpenCode session created
- agent status/thinking visible
- agent response persisted
- file changed
- command run
- Browser preview shown
- session resumable after app restart
- concurrent sessions demonstrated
- extension install/connect and enablement records available

## Non-Goals For This Freeze

- Do not call checkpoint 1 the MVP.
- Do not move documented r04 features to post-MVP through checkpoint language.
- Do not treat mocked UI as product-complete.
- Do not treat security-only progress as product viability.
- Do not expose OpenCode concepts as the primary product model.
- Do not require Pi runtime support in MVP.
- Do not include Browser downloads in MVP.

## Required Follow-Up

- Use `.agents/specs/research/implementation-checkpoint-plan.md` to sequence
  implementation.
- Keep ADRs in `docs/adr/` aligned with Browser/agent authority and extension
  enablement decisions.
- Promote only stable reusable findings into `.agents/context/`.
