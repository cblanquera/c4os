# UI Handoff Specification

Status: draft handoff
Updated: 2026-06-20

## Purpose

This document translates the current C4OS wireframes into implementation-facing
layout, behavior, and state guidance. It should be read with the functional
prototype in `wireframes/r04-single-page-app/` and the screen inventory in
`wireframes/screens.md`.

## Source Inputs

- `plans/product-interface.md`
- `plans/pegs/*.png`
- `wireframes/screens.md`
- `wireframes/r04-single-page-app/`
- `.agents/specs/mvp/requirements.md`
- `.agents/specs/mvp/acceptance.md`

## Current Review Artifact

The latest review artifact is `wireframes/r04-single-page-app/index.html`.
It is a single page app prototype:

- `/` starts on App Start, not on a review menu.
- Internal screens use hash routes such as `#chat-session`.
- Screen changes are simulated by client-side rendering, not server routing.
- The artifact is for review and handoff, not production code.

## Layout Contract

C4OS uses a desktop command-center layout with three primary regions:

1. Left navigation panel for project and session ownership.
2. Center workbench for empty prompt state or active chat session.
3. Right workspace tools panel for Browser, Files, and Terminal.

The left and right panels are resizable. The center workbench flexes between
them. Collapse controls live in the top center bar: left collapse control,
current project or session title, then right collapse control.

The app should preserve a desktop shell at narrower widths instead of becoming
a stacked mobile website. Horizontal overflow is acceptable for narrow review
widths until a separate responsive design is approved.

## Entry And Workspace State

The first usable state is App Start:

- The screen title is `Open a folder to start working`.
- The page explains that local file access, instructions, skills, runtime
  policy, approvals, and workspace persistence must be scoped to a local folder.
- Primary entry actions are Open Folder, Clone Repository, and Open Workspace
  File.
- Recent folder-backed workspaces are selectable and move into the trusted
  workspace shell.

The prompt composer is not the first state. Prompting belongs inside a trusted
workspace context.

## Shell Navigation

The left panel contains:

- Project search input.
- Projects section with an add-project action.
- Project rows with folder icon, project name, new-chat affordance, and remove
  affordance.
- Session rows nested under the active project.
- Settings entry anchored at the bottom.

Selection rules:

- In a project-level empty workspace state, the active project row is selected.
- In a chat session, the active chat session row is selected and the parent
  project row is not selected.
- Session text aligns with project-name text, while the active session
  background may extend farther left.

## Center Workbench

### Empty Workspace

The empty workspace asks:

`What should we build in c4os2?`

The composer includes:

- Attachment button.
- Approval policy chip.
- Microphone button.
- Send button.
- Branch context row showing `main`.
- Model chip showing the selected model.

The wireframe uses structural copy. Final product copy can change, but the
layout hierarchy and control ownership should remain.

### Chat Session

Chat uses messenger ownership:

- User messages align right.
- Agent messages align left.
- Agent bubbles are not full width.
- Agent messages can include a collapse/expand affordance.
- Long agent content can use Show More / Show Less.

Activity examples include:

- Completed tool call.
- Agent run waiting for sensitive action approval.

The follow-up composer stays reachable below the thread. Chat-specific locked
context can show branch and model as read-only chips.

## Composer And Model Selection

The model chip opens model/provider selection.

Provider list behavior:

- Shows provider rows such as OpenRouter and OpenAI.
- Selecting a provider drills into that provider's model list.

Model list behavior:

- Header shows the active provider.
- Back control returns to provider selection.
- Current model is visibly marked.
- Selecting a model updates the composer chip and closes the popover.

The MVP wireframe examples include Gemini 2.5 Flash, ChatGPT o4, Grok 2.0,
gpt-4.1, and a local model placeholder.

## Right Workspace Tools

The right panel tab order is fixed:

1. Browser
2. Files
3. Terminal

The active tab owns the full right-panel body. Do not add extra gear,
collapse, or secondary tabs to the right tab strip for MVP.

### Browser

Browser is a single preview surface with an address bar. It does not include a
browser tab strip in MVP.

### Files

Files has two states:

1. Explorer tree.
2. Open file/code view.

Explorer state:

- Header is the active project name.
- The tree begins directly with child folders/files.
- Do not repeat the root project folder as the first row.
- Selecting a file opens the code view.

Code view state:

- Header becomes breadcrumbs, for example `c4os2 > frontend > main.js`.
- Folder breadcrumbs return to the explorer.
- Code view fills the right-panel body and should not look like a floating card.

### Terminal

Terminal includes:

- Main read-only terminal output area.
- Resizable bottom panel for AI command preview/results.

Do not add a top terminal command label such as `>_ npm run build` in the MVP
wireframe.

## Settings

Settings uses a left settings rail and main settings content area.

Navigation order:

1. Providers
2. Models
3. Configuration
4. Plugins
5. Skills
6. MCP Servers

The settings rail starts with Back to app. It does not include a settings
search input in MVP.

### Providers

Providers manage OpenAI-compatible connections:

- Provider label must be unique.
- Existing rows show endpoint URL and saved API key status.
- Providers can be enabled or disabled.
- Add Provider opens the provider form.

Add Provider fields:

- Provider type
- Label
- API Base URL
- API key
- Auth
- Headers

The API key field belongs directly under API Base URL.

### Models

Models are fetched from enabled provider connections when available.
Each model row shows the source provider and includes enable/disable control.
The MVP wireframe does not include a separate Manage button.

### Configuration

Configuration exposes:

- Approval policy.
- Sandbox settings.
- Action to open `config.toml` externally.

Do not include a redundant config-source row.

### Plugins, Skills, And MCP Servers

The r04 wireframe includes review states for plugin marketplace, installed
skills, and MCP server management. Treat these as functional review surfaces:

- Plugin connection and marketplace dialogs are simulated.
- Skill detail dialogs are simulated.
- MCP custom server form is simulated.
- Persistence, installation, and external authentication are out of scope for
  the wireframe.

## Visual And Interaction Rules

- Keep the MVP UI neutral, utilitarian, and work-focused.
- Use familiar iconography, preferably lucide icons in implementation.
- Avoid dark theme or brand-heavy styling unless a later design phase approves
  it.
- Keep cards reserved for repeated items, dialogs, and framed tool surfaces.
- Do not put the main 3-panel app shell inside a decorative card.
- Text must not overlap or overflow controls at supported desktop review widths.
- Preserve visible keyboard focus for interactive controls.

## Simulated In The Wireframe

These behaviors are review-only in r04:

- Folder opening, cloning, and workspace file opening.
- Prompt submission.
- Provider/model persistence.
- Browser rendering.
- File editing and saving.
- Terminal command execution.
- Approval, sandbox, provider, plugin, skill, and MCP toggles.
- Plugin installation, skill execution, and MCP connection persistence.

## Implementation Notes

- Treat wireframe behavior as product intent unless later validation or user
  decision changes it.
- Promote accepted wireframe behavior into MVP requirements and acceptance
  records before freeze.
- Keep file, terminal, browser, approval, provider, plugin, skill, and MCP
  boundaries explicit in implementation.
- Preserve local-first trusted-folder scoping before allowing workspace actions.
- Preserve the distinction between project selection and active chat selection.

## Open Questions

- Final product copy is not locked.
- Responsive/mobile behavior beyond narrow desktop overflow is not approved.
- Hooks remain mentioned in older planning material but are not part of the
  r04 settings navigation.
- Production persistence behavior for plugins, skills, and MCP servers still
  needs implementation-level specification.
