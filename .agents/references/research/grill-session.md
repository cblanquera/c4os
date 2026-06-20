# Research Grill Session

Status: active
Updated: 2026-06-20

## Q-001: Concurrent Runs

Prompt: What does multiple concurrent runs mean in the first MVP?

User answer:

- Multiple agent runs may be active across different trusted project folders.
- Only one main run may be active per chat session.
- Multiple chat sessions can run at the same time.
- An agent run may eventually spawn subagents, but subagent spawning does not need to be in the first MVP as long as the model is provisioned for it.

Promotion:

- Treat concurrent runs as an MVP requirement across trusted project folders.
- Keep per-chat main-run execution single-lane.
- Provision the model for future subagent child runs without implementing subagent spawning in MVP.

## Q-002: Native Browser Scope

Prompt: What does full native Browser mean for the first MVP?

User answer:

- Full web browsing with cookies, login sessions, and downloads capability, while still isolated from C4OS secrets and privileged APIs.
- Downloads are not acceptable for MVP.
- Browser should allow local app inspection and readable generated artifacts or intake/response views.
- Browser should allow `file://` opening.

Conflict:

- Current Browser POCs only support constrained preview or raw-Wry-style isolation.
- Direct Tauri `WebviewWindow` exposed `window.__TAURI_INTERNALS__` to page content.
- Persistent cookies/login sessions plus `file://` access require a clearer trust boundary before promotion.

Status: superseded by Q-002d and Q-003.

## Q-002b: Browser Trust Boundary

Prompt: Which trust model should the MVP Browser use for browsing, file access,
and agent control?

User answer:

- Browser profile should be ephemeral per session with no persistence.
- `file://` opening should allow any file URL the Browser page tries to
  navigate to.
- Agent can use Browser automation for any page in the Browser.
- Minimum accepted floor selected: downloads disabled in MVP, cookies/login
  sessions isolated by project or mode.

Conflict:

- Ephemeral profile with no persistence conflicts with the earlier request for
  cookies/login sessions unless those sessions are explicitly non-MVP.
- Allowing any page to navigate to any `file://` URL contradicts trusted-root
  containment and would permit arbitrary local-file exposure.
- Allowing agent automation for any page without requiring bridge, credential,
  workspace-file, and shell-state isolation contradicts the Browser POC risk
  findings.
- The selected security floor omitted no privileged C4OS bridge, no provider
  credentials, no unscoped workspace files, and no shell state.

Status: superseded by Q-002d and Q-003.

## Q-002c: Browser Safety Floor

Prompt: Which Browser safety position should govern MVP?

User answer:

- Browser stays in MVP with arbitrary `file://` and any-page automation,
  accepting that this is not a trusted-root-safe design.
- `file://` policy: any file URL the page or agent tries to navigate to.
- Agent automation: any page, after user approval per domain.
- Cookies/login persistence: persistent per trusted project folder.

Consequence:

- This intentionally rejects trusted-root-safe Browser containment for MVP.
- Browser becomes a privileged local browsing surface with project-scoped
  persistence and domain-approved automation.
- This conflicts with the existing product constraint that Browser content
  must not access arbitrary workspace files, shell state, provider credentials,
  or privileged app APIs.
- This likely requires an explicit unsafe-mode product decision and a separate
  Browser ADR before freeze.

Status: superseded by Q-002d and Q-003.

## Q-002d: Browser Trust Claim

Prompt: How should C4OS describe Browser trust in MVP?

User correction:

- The prior question was too leading because it framed broader Browser access as
  unsafe instead of separating user-owned desktop authority from agent
  authority.
- C4OS is a desktop app for users who own files on their machine.
- Users should be able to browse their own local files freely.
- If a file is inside the same project folder, an agent should be able to open
  and change it freely.
- If the user did not set up Git or a revert path, that is the user's
  responsibility.
- Agent attempts to open or manipulate files outside the active project should
  require permission unless the user previously allowed that in chat.
- Users should be able to browse any public website without restrictions.
- Users should be able to log in, with session data remembered per project and
  shared by other sessions in that project.
- Agents may open and browse any public website without restrictions when it
  falls within the user's request.

Reframed interpretation:

- The Browser is not an "unsafe mode"; it is a user-owned desktop Browser with
  project-scoped session data.
- The key boundary is agent authority, not user browsing authority.
- Project-internal files and public websites are default-allowed for the agent
  when they are within the active project/request.
- Files outside the active project require explicit permission unless already
  allowed in chat.

Status: resolved by Q-003.

## Q-003: Agent Authority

Prompt: What can agents do inside the active project, outside the project, and
on the web?

User answer:

- Inside the active project folder, an agent can read and write freely when the
  user's request implies file work.
- Outside the active project folder, an agent can read and write only with
  explicit permission.
- An agent can browse any public website when it falls within the user's
  request.
- An agent can use logged-in Browser sessions when the user's request clearly
  requires it.
- C4OS should record file reads inside the project, file writes inside the
  project, outside-project permission grants, public websites visited by the
  agent, and logged-in sites used by the agent.

Promotion:

- Treat the active project folder as the default agent work boundary.
- Treat outside-project file access as permission-gated.
- Treat public and logged-in web browsing as request-scoped agent authority.
- Require audit records for file and Browser actions performed by the agent.

Status: resolved.

## Q-006: Extension Install And Connect

Prompt: Which install/connect extension flows are required in MVP?

User answer:

- MVP requires install/connect flows for plugins, skills, and MCP servers.
- Installed extensions can affect agent execution only after explicit
  per-extension enablement.
- Extension permission records must include source/provenance, granted scopes,
  workspace/project scope, data shared, runtime/tool access, enabled/disabled
  state, and audit log of extension actions.
- Skills and plugins can be invoked implicitly by "use when user wants..."
  routing or explicitly by the user.
- MCP must be explicitly invoked by the user.
- Chat prompt tagging syntax:
  - `$` for skills, for example `$superpowers`
  - `@` for plugins, for example `@gdrive`
  - `^` for MCP, for example `^gmail`

Promotion:

- Extension install/connect is MVP scope.
- Runtime impact requires explicit per-extension enablement.
- Skills/plugins may be implicit or explicit after enablement.
- MCP invocation is explicit only.
- Chat prompt tag grammar is part of the MVP interaction model.

Status: resolved.

## Q-007: Runtime Backend

Prompt: Which runtime backend should the first implementation use?

User answer:

- OpenCode first behind a thin C4OS adapter.
- Reason: OpenCode supports connectors; Pi is more raw.
- Pi remains a later adapter target only.

Promotion:

- Treat OpenCode as the first implementation backend.
- Keep the C4OS-owned adapter boundary so Pi can be added later without
  changing the product model.
- Do not expose OpenCode concepts as the primary product model.

Status: resolved.

## Q-008: First MVP Workflow

Prompt: Which first MVP workflow should freeze as the primary implementation
slice?

User answer:

- Primary workflow selected: open trusted folder, configure provider, start
  chat, agent answers with visible thinking/status.
- Result that proves the workflow worked: changed file plus command output plus
  browser preview plus resumable session record.
- Required surfaces: provider setup, OpenCode runtime, concurrent sessions,
  file read/write, agent-owned command terminal, user terminal, Browser with
  project profile.

Conflict:

- The selected workflow is a chat/status workflow, but the proof result and
  required surfaces describe a full project-change workflow.
- The next question must resolve whether the freeze slice is simple agent
  response or end-to-end project change.

Status: resolved by Q-008b.

## Q-008b: Workflow Proof Boundary

Prompt: Which workflow should freeze as the actual first MVP slice?

User answer:

- Keep both the chat/status workflow and project-change proof as one MVP slice,
  even if broader.
- Minimum acceptance for first usable MVP includes trusted folder opened,
  provider configured, OpenCode session created, agent status/thinking visible,
  agent response persisted, file changed, command run, Browser preview shown,
  session resumable after app restart, and concurrent sessions demonstrated.
- Everything shown in r04 wireframes is MVP complete.
- MVP means Minimum Viable Product: viable enough to send to a random user who
  should be able to use it as intended.
- The previous iterations were too small to be viable and overfocused on
  careful steps/security rather than product completeness.
- Checkpoint phases may exist to review progress, but they are not the MVP.
- The product is not MVP until all documented features are complete.
- Assume everything documented so far is MVP because post-MVP was purposely
  left out.

Promotion:

- Treat all documented research/r04 scope as MVP unless explicitly labeled
  post-MVP later.
- Treat checkpoints as implementation milestones, not MVP scope boundaries.
- Freeze review should validate completeness and ordering, not shrink the MVP
  below documented/r04 product scope.

Status: resolved.

## Q-009: Implementation Checkpoints

Prompt: How should implementation checkpoints be structured?

User answer:

- Implementation checkpoints should be UI shell first, then backend/runtime
  capability.
- Checkpoint 1 should prove folder trust, provider setup, OpenCode session,
  visible agent response, and persisted session.
- Checkpoint language must avoid:
  - calling checkpoint 1 the MVP
  - moving documented r04 features to post-MVP
  - treating security-only progress as product viability
  - treating mocked UI as product-complete
  - freezing implementation before all documented features are planned

Promotion:

- Use checkpoints only as sequencing and progress-review gates.
- Do not use checkpoint labels to shrink MVP scope.
- Checkpoint 1 is a progress review gate, not MVP completion.

Status: resolved.

## Q-010: Freeze Artifacts

Prompt: Which artifacts should be produced after this grill?

User answer:

- Produce an MVP freeze document.
- Produce an implementation checkpoint plan.
- Produce an ADR for Browser and agent authority.
- Produce an ADR for extension enablement and prompt tags.
- Produce a context promotion update.
- Browser and agent authority need an ADR before implementation planning.
- Promote these context items now:
  - full documented/r04 scope is MVP
  - OpenCode-first runtime decision
  - agent authority model
  - Browser project profile and request-scoped agent browsing
  - Terminal allowlist and split terminal model

Promotion:

- The grill should close into freeze artifacts, checkpoint planning, ADRs, and
  context promotion rather than more scope reduction.

Status: resolved.

## Q-004: Terminal Scope

Prompt: What does full interactive Terminal mean for the first MVP?

User answer:

- MVP uses separate terminals: a user terminal and an agent-owned command
  terminal.
- The agent may run any command inside the active project after per-command
  approval.
- Commands outside the active project are allowed only with explicit
  permission.
- C4OS should record command text, working directory, permission decisions,
  exit status, and output summary.
- Read-only commands should be allowed without asking for permission.
- The app should read a config file that defines the default command allowlist.

Promotion:

- Treat Terminal as an MVP surface with separate user-owned and agent-owned
  command channels.
- Treat read-only allowlisted commands as default-allowed.
- Treat non-allowlisted, mutating, or outside-project commands as
  approval-gated.
- Require terminal audit records for agent-owned command execution.

Status: resolved by Q-005.

## Q-005: Terminal Allowlist

Prompt: Where should the default command allowlist live, and what commands are
allowed without prompting?

User answer:

- Default command allowlist lives in app-bundled default config, overrideable by
  workspace config.
- A command is read-only only when it matches exact allowlisted command
  patterns.
- MVP default allowlist includes `pwd`, `ls`, `find`, `git status`, `git diff`,
  `git log`, and network commands such as `curl`.

Promotion:

- Treat read-only command classification as deterministic pattern matching, not
  model inference.
- Treat workspace config as able to extend or override app-bundled defaults.
- Treat `curl` and other network read commands as default-allowed when matched
  by allowlist, despite requiring network access.

Status: resolved.
