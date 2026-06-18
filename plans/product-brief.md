# Product Brief: C4OS

Greenfield restart brief. This document describes the product as if the team
were starting from scratch today, using the existing planning records only as
product intent and evidence. It does not assume the current implementation
sequence, current code shape, or completed progress items must be preserved.

## One-Sentence Summary

C4OS is a local-first desktop command center for AI-assisted work that
lets users open trusted project folders, run persistent agent sessions, manage
providers and approvals, inspect artifacts, edit files, and use safe browser
and terminal surfaces from one app-owned workspace.

## Product Thesis

AI agents are becoming useful for coding, writing, research, analysis,
operations, and documentation, but most agent experiences still force users
into one of two weak models:

- cloud-first chat tools that do not safely own local project context, files,
  approvals, credentials, or workspace state
- developer-first CLIs and runtimes that can be powerful but expose too much
  runtime detail, require terminal fluency, and leave persistence, safety,
  artifact handling, and multi-project workflows fragmented

C4OS should combine the best parts of a desktop IDE, a local-first agent
runtime, and a Codex-like session interface. The product should feel like a
workspace for real work, not a prompt box. It should be safe enough to trust
with local projects, flexible enough for technical users, and structured enough
to support repeatable sessions, approvals, artifacts, memory, and extensions.

## Product Vision

Build a desktop AI workspace where the user can:

- open or create a local project folder
- trust the folder before the app can touch files, Git, shell, skills, MCP, or
  project-scoped instructions
- start one or more persistent agent sessions
- choose model/provider settings per session
- watch agent reasoning, prework, tool activity, approvals, and generated
  outputs in context
- approve or deny sensitive local actions before they execute
- inspect, edit, and save local project files through trusted-root controls
- view generated artifacts safely, including untrusted HTML in isolated previews
- use browser and terminal surfaces that preserve app-owned safety boundaries
- resume work later with project, session, artifact, approval, and memory state
  intact

The long-term product should become the local command center for agent-assisted
work across codebases, documents, research folders, operations playbooks, and
team workspaces.

## Product Goals

- Provide a folder-first desktop AI workspace for local projects.
- Keep project files, session state, approvals, artifacts, memory, and provider
  settings under app-owned local control.
- Support multiple trusted project folders and multiple resumable sessions per
  project.
- Allow agent runs to operate concurrently without mixing approvals, outputs,
  working directories, or artifacts.
- Route sensitive actions through app-owned approval and audit boundaries.
- Support OpenAI-compatible model providers with bring-your-own-key profiles.
  Preset providers include:
  - Open Router
  - Hugging Faces
  - LiteLLM
  - OpenAI
  - Anthropic
- Preserve compatibility with useful ecosystem conventions such as `AGENTS.md`,
  `SKILL.md`, MCP, OpenCode configuration, and OpenCode plugins where practical.
- Provide safe artifact, browser, terminal, and file-explorer surfaces without
  exposing privileged app APIs to untrusted content.

## Target Users

### Primary Users

Technical power users:

- developers, founders, technical operators, and AI-heavy individual
  contributors
- comfortable with project folders, Git, terminals, providers, local files, and
  model selection
- need a stronger interface than chat and a safer interface than unconstrained
  local agent scripts

Developers and small technical teams:

- work in repositories with `AGENTS.md`, `SKILL.md`, MCP servers, project
  scripts, branch workflows, and review artifacts
- need concurrent sessions, visible approvals, file inspection, Git awareness,
  and reproducible local state
- want agent work to fit their existing repo conventions instead of requiring a
  proprietary project format

Knowledge workers with local project structure:

- write, research, document, analyze, and organize work in folders
- need persistent sessions, artifacts, provider control, and local memory
- may not use Git or shell heavily, but still benefit from trusted workspace
  boundaries and local-first persistence

### Secondary Users

Team leads and administrators:

- evaluate whether the app can safely run in organization-controlled projects
- care about policy, auditability, provider restrictions, credential handling,
  and extension trust
- need the architecture to support future organization policy without forcing a
  cloud admin console into the MVP

Plugin, MCP, and skill authors:

- want their extensions to be discoverable and trusted in a clear product UI
- need provenance, permissions, enablement, and scoping to be visible before
  runtime use

## User Problems

### Local Context Is Hard To Trust

Users want agents to understand their project, but they do not want an agent to
silently read or mutate arbitrary local files. The product must make trusted
folders explicit and keep local operations contained to those roots.

### Agent Work Is Hard To Resume

Chat transcripts, runtime events, artifacts, approvals, selected models, and
local memory often live in different places. Users need durable sessions that
can be reopened with enough context to continue real work.

### Approvals Are Too Runtime-Specific

If approvals are delegated only to the underlying runtime, the desktop product
cannot guarantee consistent safety for file writes, shell commands, MCP calls,
provider credential use, worktree changes, or destructive operations. The app
must own the approval gateway.

### Model And Provider Setup Is Fragmented

Users often use different OpenAI-compatible providers depending on task, cost,
model availability, or team policy. The app needs provider profiles, secure
BYOK storage, model discovery or manual model entry, and per-session model
selection.

### Generated Outputs Need A Workspace

Agents produce code, Markdown, diffs, images, PDFs, HTML previews, command
logs, and research notes. These should not be loose chat attachments. They need
artifact identity, safe viewing, project/session association, reopening, and
export paths.

### Power Features Need Safety Boundaries

Browser, terminal, file editing, extensions, skills, hooks, MCP, and plugins are
where agent products become useful and risky. These features must be designed
around trust, containment, provenance, and auditability from the beginning.

## Core User Flow

1. The user opens, creates, clones, or selects a local project folder.
2. The user trusts the folder before local file, Git, shell, instruction, skill,
   or approval-scoped workflows become available.
3. The user starts or resumes a session from the project workspace.
4. The user selects a provider/model profile and submits prompts through the
   composer.
5. Agent activity streams into the session view with visible status, tool
   activity, approvals, generated artifacts, and runtime state.
6. The user approves, denies, remembers, or audits high-impact actions.
7. The user inspects or edits project files, views artifacts, and uses approved
   browser or terminal surfaces where available.
8. The workspace, session metadata, approvals, artifacts, provider references,
   and local memory remain available for later continuation.

## Product Principles

### Local-First By Default

The app should store project metadata, session metadata, approvals, artifacts,
audit logs, local memory, and workspace state on the user's machine by default.
Cloud sync and organization policy can be future layers, not MVP assumptions.

### Project-First, Folder-Trusted

The app should lead users to open, create, clone, or select a project folder
before prompting. Local file, Git, shell, instruction, skill, and approval
workflows should require a trusted folder.

### App-Owned Product Model

The user should interact with workspaces, project folders, sessions, agent runs,
approvals, artifacts, providers, files, browser surfaces, terminal surfaces, and
settings. Underlying runtimes such as OpenCode and Pi should remain implementation
details behind an adapter.

### Explicit Trust Before Execution

High-impact actions must pass through app-owned policy before execution. This
includes shell commands, file writes/deletes, MCP mutation, credential use,
networked tools, worktree mutation, share/export, and destructive actions.

### Standards-Compatible, Not Proprietary-First

The app should respect useful ecosystem conventions such as `AGENTS.md`,
`AGENTS.override.md`, `SKILL.md`, MCP, OpenCode/Pi-compatible config, and
OpenCode/Pi-compatible plugins where practical. It should avoid inventing
proprietary replacements without a strong product reason.

### Safe Surfaces Over Raw Power

Browser, terminal, artifact preview, and file editing should be useful, but
they must not expose privileged app APIs or arbitrary local authority to
untrusted content or renderer code.

### Durable Work, Lightweight Files

Workspace files should be portable descriptors, not full archives. Private
state such as transcripts, artifacts, audit logs, secrets, and local operational
state should stay in app-owned local storage or explicitly exported formats.

## Core Experience

### First Run

On first launch, the user sees a folder-first empty state. The primary actions
are:

- open an existing folder
- create a new local folder-backed workspace
- clone a repository
- open a recent trusted workspace or folder

The prompt composer is not enabled until a project folder is present and
trusted. The first-run experience should explain trust through UI state and
permissions, not long instructional text.

### Main Workspace

The main app should use a Codex-like desktop layout:

- left panel for projects, chats, sessions, search, scopes, and navigation
- central session transcript and prompt composer
- contextual right or secondary surfaces for files, artifacts, approvals,
  provider settings, runtime state, browser, terminal, and inspectors
- persistent status areas for active runs, pending approvals, model/provider
  state, trusted root state, and runtime health

The first usable screen after trusting a folder should be the active workspace,
not a dashboard or marketing-style landing page.

### Session Flow

A typical session should work like this:

1. User selects a trusted project and starts a new chat/session.
2. User chooses provider/model, branch/worktree preference where applicable,
   approval mode, and attachments or tags.
3. User submits a prompt.
4. The session shows agent prework, thinking/status, tool activity, and pending
   approvals before final response.
5. Sensitive actions are routed to approval controls.
6. Generated artifacts, file references, terminal output, browser results, and
   runtime events appear in context.
7. The user can continue, inspect, edit, export, or resume later.

### Prework And Response Rendering

The chat session should show when the agent is thinking and what prework it is
doing before sending the final response. Prework should be grouped and collapsed
after the final response is sent. The interaction should feel polished and
Codex-like, with typing or graceful text reveal that makes progress visible
without turning the transcript into noisy logs.

### Prompt Composer

The prompt composer should support:

- disabled state until a trusted folder is available
- text prompt entry
- file attachments
- branch selection or new branch creation
- approval mode selection
- speech-to-text
- inline tagging for skills, plugins, agents, models, MCP servers, and related
  scope-aware entities
- clear visibility into which project, model, branch, approval profile, and
  scopes will apply to the run

## Feature Areas

### Workspace And Project Management

The app needs a local project registry that tracks:

- project display name
- local path
- trust state
- recent sessions
- runtime configuration references
- provider/model preferences
- optional Git metadata
- workspace membership

Users should be able to add, inspect, remove, and switch between multiple local
projects. Multiple trusted folders should be manageable without moving files or
forcing a repository structure.

Workspace files should use a `.c4os` descriptor model. A workspace file
can reference folders, display names, workspace settings, enabled extension
references, default provider/model preferences, and non-secret IDs. It must not
contain raw secrets, full transcripts, artifact archives, or private operational
state by default.

### Sessions And Agent Runs

Each project should support multiple sessions. A session should include:

- transcript and message index
- run status
- selected agent
- selected model/provider
- approval state and audit references
- artifact references
- local memory references
- resume metadata
- runtime references
- branch/worktree state where applicable

The product should support concurrent agent runs without mixing approvals, tool
output, working directories, artifacts, or runtime events. Users should be able
to see active status, cancel runs, recover from errors, and resume sessions.

### Runtime Orchestration

The MVP runtime strategy consider the use of OpenCode Runtime or Pi Runtime 
behind an app-owned adapter, assuming the runtime provides stable enough 
session control, streaming events, tools, permissions, and configuration APIs.

```
  C4OS AI Harness
       ↓
OpenCode/PI Runtime
       ↓
    LLM API
```

The product should own:

- user-facing session identity
- workspace persistence
- artifact identity
- approval policy
- provider settings
- local memory
- runtime lifecycle supervision
- runtime recovery and error reporting

The runtime adapter should hide OpenCode/Pi-specific concepts unless exposing them
is necessary for advanced settings or compatibility inspection.

### Approval And Security Controls

The approval layer should support:

- allow, ask, and deny modes
- per-project defaults
- per-session overrides
- scoped remembered rules
- pending approval queue
- clear action descriptions
- affected files, commands, tools, providers, or roots
- audit log entries for approved, denied, remembered, cancelled, and failed
  actions

The app-owned policy engine should evaluate sensitive operations before the
runtime or tool executes them. The runtime can surface its own permission
requests, but it should not be the only enforcement layer.

### Provider And Model Management

The app should support OpenAI-compatible provider profiles with BYOK storage.
Initial built-in profile types should include:

- OpenAI
- OpenRouter
- Hugging Face router
- LiteLLM proxy
- custom OpenAI-compatible endpoint

Provider settings should include:

- enable/disable state
- credential setup
- secure key storage status
- base URL where applicable
- model discovery or manual model entry
- model enable/disable state
- model search
- per-session model selection
- tested connection status
- hidden provider inputs auto-filled for built-in provider types on save

Raw provider keys must be stored only in OS keychain or the selected secure
storage abstraction.

### File Explorer And Editor

The File Explorer should feel familiar to users of VS Code:

- trusted-root file tree
- recursive folder navigation
- folders with children expand and collapse
- hidden files and folders visible except `.git`
- known file types use recognizable icons
- compact metadata without turning rows into a crowded table
- Git-aware colors where reliable Git state is available
- safe degradation when Git state is unavailable

The editor should support:

- file click switches the main view to an editor
- content loads from trusted roots only
- code highlighting based on file type
- Markdown/code rendering where appropriate
- clickable breadcrumbs back to folders
- dirty state
- explicit Save and Revert
- New File and New Folder
- Rename
- guarded Delete with exact-path confirmation
- external change detection
- Reload and Keep Draft controls
- multi-tab editing
- active-file literal search with next/previous navigation

The first principles boundary is that file editing is user-controlled. Agent
initiated mutation should require separate approval-boundary work.

### Artifacts And Preview Surfaces

Generated artifacts should be first-class outputs, not incidental chat blobs.
The app should store and display:

- Markdown
- plain text
- code
- diffs
- generated HTML
- images
- PDFs
- downloadable files
- command logs where appropriate
- research or analysis outputs

Artifact records should preserve project/session association, creation time,
originating run, export/reveal options, and safe viewer type.

Generated or untrusted HTML must render in an isolated preview surface without
privileged app APIs. External web browsing should be visually and technically
separate from local artifact previews.

### Browser Surface

The product should eventually provide a working general Browser tab for
approved local preview and web URLs.

The Browser surface must not expose provider credentials, arbitrary workspace
files, shell state, or privileged app APIs to page content. Native Browser
implementation should wait until the team has proven cross-platform raw Wry
behavior and confirmed that product Browser content has no privileged Wry IPC
handler.

Greenfield implementation should design the browser permission model first,
then promote native WebView work only after isolation proof.

### Terminal Surface

The product should provide a working general Terminal tab for trusted projects.
The terminal should be backend-owned, not a renderer-owned shell.

Terminal design requirements:

- trusted-root cwd validation
- approval-gated launch for high-impact commands
- sanitized environment
- PTY spawn behind approved launch plans
- backend-owned process/session lifecycle
- bounded renderer event transport and backpressure
- input, output, resize, cancel, disconnect, cleanup, and exit events
- audit records for terminal lifecycle actions
- clear separation between user command input and agent/runtime output

Future terminal expansion should be explicit product priority. SSH, remote
shells, containers, multiplexing, agent auto-run, and arbitrary renderer shell
spawn should not appear as accidental scope.

### Left Panel And Navigation

The left panel should help users move through work quickly:

- project/workspace navigation
- chat/session list
- new chat creation with visible label update
- search across chat session items
- visible plugin/scoping context
- chat removal
- recent work access
- active run indicators
- pending approval indicators

This should remain a work surface, not a decorative dashboard.

### Settings And Scoping

Settings should give users control over:

- providers
- models
- approval modes
- saved approval rules
- configuration settings
- skills
- MCP servers
- hooks
- agents
- runtime configuration
- customizations

Scoping should be visible: users should understand whether a setting applies to
the app, workspace, project, session, runtime, provider, extension, or model.
Runtime configuration scoping should include OpenCode and `pi`.

### Standards, Extensions, And Marketplace Readiness

The product should discover, display, and respect compatible instruction and
skill files, including:

- `AGENTS.md`
- `AGENTS.override.md`
- `SKILL.md`
- `.opencode/skills`
- `.pi/skills`
- `.claude/skills`
- `.agents/skills`

The post-MVP extension system should provide a shared inventory for MCP
Servers, Skills, Plugins, Hooks, Agents, and related scoped surfaces. The
inventory should show:

- provenance
- install source
- requested permissions
- trust state
- enabled state
- risk/approval requirements
- scope
- runtime impact

Extensions should be disabled by default before they can affect runtime
execution, model context, tools, hooks, or app-owned state. Harmless MCP
connection should land before executable plugin loading or automatic hook
behavior.

Marketplace readiness is an architecture goal, not MVP scope. The app should
not require a proprietary package format before local install, trust, signing,
and permission boundaries are proven.

### Local Memory

Local memory should be app-owned and scoped to workspace, project, or session.
It can include:

- session summaries
- pinned facts
- retrieval hooks
- project preferences
- recurring task context
- user-approved durable notes

Memory should remain separate from raw provider state and runtime-local session
storage. Users should be able to understand what is remembered and where it is
scoped.

### File Explorer And Editor Expansion

Once the MVP proves trusted browsing and safe persistence, expand into a
VS Code-like editor surface with create, rename, delete, conflict handling,
multi-tab editing, search/navigation, and eventually richer editor features.

### Native Terminal

Promote terminal work only through backend-owned lifecycle, approval-gated
launch, sanitized environment, PTY management, renderer transport, and audit
records.

### Native Browser

Promote browser work only after cross-platform isolation proof. The native
Browser surface must not expose privileged app APIs or secrets to page content.

### Extension Inventory

Build a disabled-by-default inventory for MCP Servers, Skills, Plugins, Hooks,
Agents, and runtime customizations. Start with harmless MCP connection before
executable plugins or automatic hooks.

### Settings And Scoping

Make provider, model, skill, MCP, hook, agent, runtime, and customization scopes
visible and manageable.

### Prompt And Chat Polish

Improve attachments, branch selection, approval modes, speech-to-text, inline
tagging, prework rendering, and Codex-like response feel.

## Data Model Concepts

The greenfield product should model at least these entities:

- Workspace
- Workspace file
- Project folder
- Trust record
- Session
- Agent run
- Transcript message
- Runtime event
- Approval request
- Approval decision
- Saved approval rule
- Artifact
- Provider profile
- Model profile
- Credential reference
- Local memory record
- Extension inventory item
- Skill reference
- MCP server reference
- Plugin reference
- Hook reference
- Browser state record
- Terminal session record
- Audit log entry

Secrets should be stored as secure references, not raw values in general
application tables or workspace files.

## Security And Privacy Requirements

- Local-first storage by default.
- Trusted-root containment for local file operations.
- Explicit approval for high-impact actions.
- App-owned policy before runtime execution.
- Secure OS storage for raw API keys.
- Isolated preview for untrusted/generated HTML.
- Browser content cannot access privileged app APIs.
- Renderer cannot spawn arbitrary shells directly.
- Terminal process handles stay in backend-owned state.
- Audit logs record sensitive action decisions.
- Workspace descriptor files avoid transcripts, raw secrets, and full artifact
  archives by default.
- Extension execution requires provenance, trust, permissions, and enablement.

## Success Metrics

### Product Usability

- A new user can open and trust a project folder without reading external docs.
- A user can start a session, choose a model, submit a prompt, approve an
  action, and inspect the result in one flow.
- A user can leave and return to a session with transcript, artifacts, selected
  model, and approval history intact.

### Trust And Safety

- Sensitive file, shell, MCP, credential, and destructive actions are blocked or
  approval-gated before execution.
- Untrusted generated HTML cannot access privileged app APIs.
- Raw provider keys never appear in workspace files or normal app data.
- File and terminal actions cannot escape trusted roots through traversal.

### Technical Viability

- Runtime lifecycle can start, stop, recover, and report errors reliably.
- Provider profiles can connect to at least one real OpenAI-compatible provider.
- Artifact storage and viewers work across the frozen MVP artifact types.
- Sessions can run concurrently without cross-run approval or output leakage.

### Workflow Value

- Users can complete meaningful local project work without leaving the app for
  basic file inspection, approvals, artifacts, provider setup, and transcript
  continuity.
- The app feels like a workspace, not a one-off chat window.

## Risks

### Runtime Coupling

If the product leaks OpenCode/Pi concepts too directly, future runtime changes
will be expensive and the UI will feel like a wrapper instead of a workspace.
Mitigation: keep OpenCode/Pi behind an adapter and maintain app-owned product
language.

### Security Boundary Drift

Browser, terminal, artifact preview, file editing, and extensions can quietly
expand local authority. Mitigation: require explicit trust records, approval
policy, isolated surfaces, and audit logs for each capability.

### Provider Complexity

Provider-native APIs differ widely. Supporting them all directly can consume
MVP scope. Mitigation: use OpenAI-compatible profiles first and defer direct
provider-native APIs.

### Extension Risk

Plugins, hooks, MCP servers, and skills have different execution models.
Mitigation: use one inventory/trust/provenance model while keeping execution
mechanisms separate.

### UI Scope Creep

A desktop AI workspace can sprawl into IDE, browser, terminal, marketplace,
memory manager, and admin console all at once. Mitigation: ship a folder-first
MVP with clear post-MVP promotion gates.

### Persistence Confusion

Users may expect workspace files to carry all state. Mitigation: define
workspace files as portable descriptors and provide explicit export/import later
for full state migration.
