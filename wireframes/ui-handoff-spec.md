# C4OS UI Handoff Specification

Status: draft handoff
Updated: 2026-06-20
Primary artifact: `wireframes/r04-single-page-app/index.html`

This document explains the r04 C4OS wireframes as an implementation handoff.
It is written for an agent or engineer who has not opened the r04 folder. The
goal is to remove guesswork about layout, state, interactions, simulated
behavior, and implementation boundaries.

## 1. How To Use This Document

Use this document when translating the r04 wireframes into product
implementation, acceptance criteria, frontend tasks, or detailed UI specs.

Read it in this order:

1. Read the product frame and source list.
2. Read the normative versus illustrative convention.
3. Read the global shell contract.
4. Read the screen sections for the surface being implemented.
5. Check interaction, simulation, and open-question sections before turning
   wireframe behavior into implementation work.

This document is authoritative for what the wireframes currently express. It is
not a production architecture plan and does not claim backend persistence,
provider calls, file writes, terminal execution, plugin installation, skill
execution, or MCP connectivity has already been implemented.

## 2. Normative Versus Illustrative Content

The wireframes include placeholder data and mock content so reviewers can see
layout, hierarchy, state, and interaction intent. Unless a value is separately
promoted into requirements, acceptance criteria, configuration, or product copy,
concrete example values are illustrative only and must not be hardcoded.

Normative requirements:

- Layout regions and ownership.
- Screen states and navigation relationships.
- Interaction patterns and control placement.
- State distinctions, such as project selected versus active chat selected.
- Trust boundaries and simulation boundaries.
- Accessibility expectations.
- Required product surfaces, such as Browser, Files, Terminal, Providers,
  Models, Configuration, Plugins, Skills, and MCP Servers.

Illustrative-only content:

- Project, workspace, and session names.
- File and folder names.
- Provider, model, plugin, skill, and MCP server examples.
- Terminal output, code snippets, URLs, branch names, and API values.
- Placeholder copy and sample message text.
- Badges, statuses, and row labels used only to demonstrate shape.

When a section lists concrete values, treat them as r04 examples unless the
section explicitly says the value is normative. Implementation should bind real
values from product state, configuration, local workspace data, or finalized
copy sources.

Wireframe versus final frontend guard:

- Implement the behavior, layout relationships, state model, navigation, and
  control intent described here.
- Do not treat the r04 monochrome styling, placeholder labels, sample records,
  sample copy, or example data as final production frontend requirements.
- Apply the production design system, theme tokens, final copy, and real data
  sources when they exist.
- If production visual direction is not yet defined, keep the interface clean
  and functional, but mark final theming as unresolved instead of inventing a
  permanent brand treatment from the wireframe.

## 3. Source Material

The handoff is derived from:

- `wireframes/r04-single-page-app/`
- `wireframes/screens.md`
- `plans/product-interface.md`
- `plans/pegs/*.png`
- `.agents/specs/mvp/requirements.md`
- `.agents/specs/mvp/acceptance.md`

The r04 prototype is the latest review target. Earlier revisions remain useful
as design history, but r04 is the current handoff baseline.

## 4. Product Frame

C4OS is a local-first desktop command center for agentic project work. The UI
centers on trusted project folders, chat sessions, local files, terminal
output, browser previews, provider/model configuration, plugin surfaces, skill
surfaces, and MCP server connections.

The interface should feel utilitarian and work-focused. It should not feel like
a marketing page, decorative dashboard, or generic SaaS landing screen. The
wireframe uses neutral grayscale styling because r04 is about layout,
behavior, and reviewable functionality rather than final brand styling.

The main product promise visible in the wireframe is controlled local work:
before a user prompts the agent, the application asks the user to scope work to
a trusted local project folder.

## 5. r04 Artifact Contract

The r04 artifact is a single page app prototype:

- `index.html` is the only HTML entry point.
- `/` opens App Start.
- There is no starting wireframe menu.
- Screen states are addressed by hash routes such as `#chat-session`.
- Hash changes render different static screen states in the same page.
- The prototype uses JavaScript to compose DOM for review.
- The prototype is not production code and can be deleted after handoff.

The route list is:

| Route | Screen |
| --- | --- |
| `#app-start` | App Start |
| `#new-session` | New Session |
| `#chat-session` | Chat Session |
| `#providers-popover` | Provider selector popover |
| `#models-popover` | Model selector popover |
| `#file-explorer` | Files tool explorer state |
| `#file-editor` | Files tool editor state |
| `#terminal` | Terminal tool state |
| `#settings-providers` | Settings Providers |
| `#settings-add-provider` | Add Provider form |
| `#settings-models` | Settings Models |
| `#settings-configuration` | Settings Configuration |
| `#settings-plugins` | Settings Plugins |
| `#settings-skills` | Settings Skills |
| `#settings-mcp` | Settings MCP Servers |

Unknown or stale routes fall back to App Start.

## 6. Global Layout Model

C4OS uses a desktop shell with three functional regions:

1. Left navigation panel.
2. Center workbench.
3. Right workspace tools panel.

This is not a responsive card layout. Do not place the app shell inside a
decorative outer card. Do not stack the shell into mobile sections unless a
future responsive design explicitly approves that behavior. At narrow desktop
review widths, preserving the desktop shell with horizontal overflow is
acceptable.

### 6.1. Left Navigation Panel

The left panel owns project and session navigation. It includes:

- Project search field.
- Projects section label.
- Add Project icon button.
- Project rows.
- Nested chat session rows.
- Settings entry fixed at the lower part of the panel.

Project row anatomy:

- Folder icon.
- Project name.
- New chat pencil icon.
- Remove project trash icon.

Illustrative project row examples in r04 are:

- `suite`
- `c4os2`
- `techops`
- `ingest`
- `chrisai`

The active-project value is illustrative. r04 uses `c4os2` as the active
project example. Illustrative session examples under that project are:

- `Locate Tauri integration`
- `Draft wireframes`

Selection rules:

- In project-level states, the active project row is selected.
- In chat-session state, the active chat session row is selected.
- When a session is active, the parent project row is not highlighted.
- Session row text aligns with project-name text.
- Session highlight can extend farther left than the text.

### 6.2. Center Workbench

The center region changes by route:

- App Start uses a two-column onboarding layout.
- New Session uses an empty workspace prompt.
- Chat Session uses a scrollable message thread and docked follow-up composer.
- Provider/model popover routes use the new-session center with an anchored
  selection popover.

The center top bar exists in shell states, not App Start. It includes:

- Left panel collapse button.
- Current project or session title.
- Right panel collapse button.

The top title reflects current state. r04 examples are:

- `c4os2` for project-level shell states.
- `Locate Tauri integration` for the chat-session state.

### 6.3. Right Workspace Tools Panel

The right panel owns workspace tools. Its tab order is fixed:

1. Browser
2. Files
3. Terminal

The active tab controls the full body of the right panel. Do not add secondary
tabs, a browser tab strip, a gear tab, or a panel-management strip inside this
tab bar for MVP.

The right panel has three screen-state families:

- Browser preview.
- Files explorer/editor.
- Terminal output/results.

### 6.4. Resizing And Collapse

Left and right panels are resizable. Resize handles live between the side
panels and center workbench.

Resize behavior:

- The left resize handle controls the width of the left navigation panel.
- The right resize handle controls the width of the right workspace tools panel.
- Dragging a handle changes only that side panel's width.
- The center workbench absorbs remaining horizontal space.
- Dragging should stop at minimum and maximum widths so no panel becomes
  unusable and the center workbench is not squeezed below a practical minimum.
- The wireframe demonstrates the interaction but does not define persisted
  widths. Production can decide whether resized widths persist per window,
  workspace, or session.
- Resize handles are separators, not buttons. They should use resize cursor
  behavior and support keyboard-accessible equivalents in production.

Collapse behavior is simulated:

- Left collapse hides the left panel and its resize handle.
- Right collapse hides the right panel and its resize handle.
- If both are collapsed, the center workbench can occupy the whole shell.
- Collapse is different from resize. Collapse is an intentional hidden/visible
  state; dragging a panel to a small width should not be the only way to hide
  it.
- Collapsing a panel should preserve enough state to restore it when the user
  expands that side again.
- The top bar collapse buttons are the visible controls for collapse. Resize
  handles are only for width adjustment.

Implementation should preserve resize boundaries so side panels cannot compress
the center workbench into unusable width.

## 7. App Start Screen

Route: `/` or `#app-start`

App Start is the first screen. It replaces the earlier review menu.

The left copy column contains:

- Kicker: `No trusted project folder`
- Heading: `Open a folder to start working`
- Body copy explaining that local file access, instructions, skills, runtime
  policy, approvals, and workspace persistence must be scoped before prompting.
- Three actions: Open Folder, Clone Repository, Open Workspace File.

The right column contains `Recent Folder-Backed Workspaces`.

Illustrative recent workspace rows:

- `c4os2` with badge `Trusted`
- `agent-lab` with badge `2 roots`
- `docs-workbench` with badge `Saved`

Clicking a recent workspace enters the trusted project shell. The example
workspace names and badges are placeholders. The wireframe does not implement
real folder opening, clone flows, or workspace-file parsing. Those actions are
visual affordances only.

Implementation implication: prompting is not allowed before trusted workspace
selection. The first active chat state must be downstream of folder-backed
workspace scope.

## 8. New Session Screen

Route: `#new-session`

New Session is the trusted project shell with no active thread content.

The shell state includes:

- Left navigation panel with the active project selected.
- Center top bar titled with the active project name.
- Empty center prompt asking what to build in the active project.
- Prompt composer.
- Right workspace tools panel in Browser state.

The empty prompt is intentionally centered. It is a workspace-ready prompt, not
a first-run onboarding screen. The r04 copy `What should we build in c4os2?`
is placeholder copy and must not be hardcoded.

## 9. Prompt Composer

The prompt composer appears in empty workspace and chat follow-up contexts. It
has these visible parts:

- Editable prompt surface.
- Hidden file input used by the attachment affordance.
- Attachment preview area when files are attached.
- Lower control row.
- Context strip.
- Optional popovers for approval policy and branch selection.
- Optional provider/model popover beside the composer.

Illustrative empty-state placeholder: `Do anything`

Illustrative chat follow-up placeholder: `Ask for follow-up changes`

### 9.1. Composer Control Row

The control row contains:

- Attach File icon button.
- Approval policy chip.
- Spacer.
- Microphone icon button.
- Send Prompt icon button.

The approval chip initially reads `Approve for me`. The approval popover
contains:

- Ask for approval
- Approve for me

The approval popover controls how the agent should handle sensitive or
high-impact actions for the next prompt/session context.

Expected behavior:

- Clicking the approval chip opens a small popover anchored to the chip.
- Selecting `Ask for approval` means the agent should stop and request user
  confirmation before sensitive actions.
- Selecting `Approve for me` means the user is allowing the agent to proceed
  through supported approval points without asking every time, subject to
  product policy and sandbox limits.
- Selecting an option updates the chip label and closes the popover.
- The popover is local to the composer context in the wireframe. It does not
  replace the global approval policy settings screen.
- In production, this control must still respect any stricter global policy,
  workspace policy, or sandbox restriction.

### 9.2. Context Strip

The context strip shows branch and model context. r04 examples are:

- Branch chip: `main`
- Model chip: `gemini-flash-2.5`

In active chat context, branch and model become read-only chips. The wireframe
signals that branch/model selection is locked for that chat.

The branch popover options in non-read-only composer state are illustrative:

- `main`
- `feature/trust-shell`
- `+ Create branch`

The branch popover controls which Git branch or branch intent the next prompt
should use.

Expected behavior:

- Clicking the branch chip opens a popover anchored to that chip.
- Existing branch rows represent selectable branch context.
- Selecting an existing branch updates the branch chip and closes the popover.
- `+ Create branch` represents a future create-branch flow. It is visual in
  r04 and should not be hardcoded as a literal branch name.
- In an active chat, branch context is read-only because the session is already
  scoped to a branch. The read-only chip communicates that later prompts in the
  same chat should not silently change branch context.
- Production implementation should decide whether branch choice is per prompt,
  per chat, or per workspace, but the UI must make the active branch context
  visible before submission.

### 9.3. Attachments

The attachment control opens a file picker in the prototype. Selected files are
shown as chips with:

- File icon.
- File name.
- File size.
- Remove control.

Attachments provide extra prompt context. They are not generic file uploads in
the wireframe; they are context items the user intentionally attaches to the
next prompt.

Expected behavior:

- Clicking Attach File opens a file picker.
- Multiple files can be selected.
- Each selected file appears as an attachment chip above the prompt body.
- Each chip shows enough identity for the user to recognize the file.
- Removing a chip detaches that file from the next prompt.
- Removing an attachment should not delete or modify the underlying file.
- Attachments should be included in the prompt context only while the composer
  still shows them.
- Production implementation must treat attachment selection, file access, and
  file scoping as trust-sensitive workspace behavior.
- If an attached file is outside the trusted workspace, production behavior must
  clearly decide whether to reject it, request approval, copy it into scoped
  context, or mark it as external.

## 10. Provider And Model Selection

Routes:

- `#providers-popover`
- `#models-popover`

Both routes render the New Session shell with a composer popover anchored near
the model chip.

### 10.1. Provider Popover

The provider popover title is `Providers`. Its subtitle is `Select source`.

Illustrative provider choices:

- OpenRouter
- OpenAI
- LiteLLM Local

Each provider row drills into the model list.

### 10.2. Model Popover

The model popover shows a back bar with the selected provider, currently
`OpenRouter`. The back control returns to provider selection.

Illustrative visible models:

- Gemini 2.5 Flash
- ChatGPT o4
- Grok 2.0

The current model is visibly marked with a check icon. Provider and model names
are placeholders unless supplied by real configuration/provider data. Selecting
a model closes back to the new-session shell in the prototype. Real persistence
and provider fetching are not implemented in the wireframe.

Purpose and expected behavior:

- The model chip chooses which model will answer the next prompt or session.
- Models are grouped by provider so users understand where the model comes
  from.
- Opening the model selector can start at provider selection or at the active
  provider's model list. r04 demonstrates both states as separate routes.
- The provider list is a drill-in menu. Selecting a provider opens that
  provider's model list.
- The model list has a back control so the user can return to provider
  selection without closing the whole selector.
- The active/current model is visibly marked.
- Selecting a different model updates the composer model chip and closes the
  selector.
- If the chat is already active and model context is locked, the model chip is
  read-only and should not open the selector.
- Production implementation should source provider/model rows from configured
  provider connections, not from r04 placeholder values.

## 11. Chat Session Screen

Route: `#chat-session`

Chat Session represents an active session inside the trusted project.

The r04 active session title `Locate Tauri integration` is illustrative.

The visible thread uses illustrative content to demonstrate four item types:

1. User message.
2. Agent response.
3. Tool Call activity card.
4. Agent Run activity card waiting for sensitive action approval.

### 11.1. Message Ownership

User messages align right and leave space on the left. Agent messages align
left and leave space on the right. This ownership model must be preserved.

Agent bubbles are not full width. The layout should make it clear who owns each
message without relying on color alone.

### 11.2. Agent Message Controls

Agent responses can include:

- Icon-only collapse/expand affordance.
- Show More / Show Less content disclosure.

The first agent message includes hidden extra detail. Show More expands the
extra content inside the same message and changes to Show Less. Show Less
collapses it.

### 11.3. Activity Cards

Activity cards represent agent work and approval boundaries. The wireframe
shows:

- Tool Call with status `Completed`.
- Agent Run with label `Sensitive action waiting`.
- Review Approval action.

Implementation should treat these as separate structured event surfaces, not
plain message text.

### 11.4. Follow-Up Composer

The chat composer is docked under the thread and remains reachable. Its context
chips are read-only because the chat has locked branch and model context.

## 12. Browser Tool

Default right-panel state for New Session and Chat Session.

The browser tool contains:

- Address bar with a local preview URL.
- Preview surface.
- Helper text indicating that browser tabs are out of scope.

The r04 URL, title, and helper text are placeholders. MVP browser behavior is
one preview surface. Do not add browser tabs unless a future product decision
expands the scope.

## 13. Files Tool

Routes:

- `#file-explorer`
- `#file-editor`

Both render the shell with the right panel in Files state.

### 13.1. File Explorer

The file explorer heading is the active project name. r04 uses `c4os2` as the
placeholder project name.

Illustrative rows:

- `backend` folder
- `frontend` folder
- `main.js` file
- `index.html` file
- `tests` folder

The root project folder is not repeated as the first row. The tree starts with
items inside the active project.

### 13.2. File Editor

The editor state replaces the file tree with breadcrumbs and a code pane.

Illustrative breadcrumbs:

- c4os2
- frontend
- main.js

Illustrative code:

```js
import { startWorkspace } from "./runtime";

const project = "c4os2";
const trustedRoot = true;

startWorkspace({ project, trustedRoot });

// Code view fills the full tool body.
```

The code snippet is mock content. The code pane fills the right-panel body. It
should not look like a decorative card. Line numbers are visible. Folder
breadcrumbs return to explorer state in the prototype.

## 14. Terminal Tool

Route: `#terminal`

The Terminal state contains:

- Read-only terminal output area.
- Horizontal resize handle.
- Bottom AI command preview/results panel.

Illustrative terminal output:

```text
$ npm run dev
ready in 614ms
local preview available at 127.0.0.1:3000
```

The terminal output and helper copy are placeholders. The bottom panel title
expresses the intended function: AI command preview/results. The bottom panel
is read-only in the wireframe and the resize handle sits on the top boundary.

The wireframe does not implement terminal input, command execution, or process
management. It only establishes the layout for terminal output and AI command
preview/results.

Purpose of the bottom panel:

- The top terminal area represents terminal output or a live terminal stream.
- The bottom panel represents agent-owned command preview/results, separate
  from the terminal stream.
- The bottom panel is where the app can show what the agent proposes to run,
  why it wants to run it, approval state, pending command details, execution
  results, or command-related explanation.
- Keeping this panel separate prevents agent command review from being confused
  with raw terminal output.
- The bottom panel is read-only in r04. Production may add review actions, but
  command editing or execution controls must be designed deliberately.
- The horizontal resize handle lets the user allocate more or less space to
  command preview/results without leaving the Terminal tab.

## 15. Settings Shell

Settings screens use a two-column settings layout:

- Left settings navigation rail.
- Main settings content.

The left rail contains:

- Back to app.
- Kicker: Settings.
- Providers.
- Models.
- Configuration.
- Divider.
- Plugins.
- Skills.
- MCP Servers.

There is no settings search input. Back to app returns to the project shell.

## 16. Settings Providers

Route: `#settings-providers`

The Providers screen manages OpenAI-compatible provider connections.

Header:

- Title: Providers
- Summary: `Manage OpenAI-compatible provider connections. Labels must be unique.`
- Add Provider primary action.

Illustrative provider rows:

- OpenRouter - Personal
- OpenAI - Work
- LiteLLM Local

Each provider row shows:

- Provider label.
- Endpoint URL.
- API key saved status.
- Edit button.
- Enabled switch.

Provider labels, URLs, and saved-key statuses are placeholders. The wireframe
does not persist provider toggles or edits.

## 17. Add Provider

Route: `#settings-add-provider`

The Add Provider form saves an OpenAI-compatible connection profile.

Fields:

- Provider Type select.
- Label input.
- API Base URL input.
- API Key password input.
- Auth select.
- Headers textarea.

Important ordering rule: API Key belongs directly under API Base URL.

Actions:

- Save Provider.
- Cancel.

Both actions return to Providers in the prototype. Real validation, secure key
storage, and provider connection testing are outside the wireframe.

## 18. Settings Models

Route: `#settings-models`

The Models screen presents models fetched from enabled provider connections.

Illustrative model rows:

- Gemini 2.5 Flash from OpenRouter - Personal, marked Current.
- ChatGPT o4 from OpenRouter - Personal, marked Enabled.
- Grok 2.0 from OpenRouter - Personal, marked Enabled.
- gpt-4.1 from OpenAI - Work, marked Enabled.
- local/coder-small from LiteLLM Local, marked Enabled.

Model names and provider labels are placeholders. Each row has an enable
switch. The wireframe does not include a separate Manage button for models.

## 19. Settings Configuration

Route: `#settings-configuration`

The Configuration screen exposes runtime control areas.

Header summary:

`Configure approval policy and sandbox settings.`

Header action:

- Open config.toml externally.

Cards:

- Approval Policy: choose when the app asks before high-impact actions. Action
  label: `On request`.
- Sandbox Settings: choose how much agents can do when running commands.
  Action label: `Workspace write`.

The wireframe intentionally avoids a redundant config-source row.

## 20. Settings Plugins

Route: `#settings-plugins`

The Plugins screen presents a plugin catalog and plugin connection flow.

Header:

- Title: Plugins.
- Summary: `Manage installed plugins and extension surfaces.`

Catalog controls:

- Search plugins input.
- Filter button labeled `Built by C4OS`.
- Filter menu with `+ Add Marketplace`.

Illustrative plugin cards:

- GitHub
- Slack
- Notion
- Linear
- Gmail
- Google Calendar
- Google Drive
- Figma
- Vercel

Plugin names and descriptions are placeholders for catalog shape. Each card
shows a logo mark, plugin name, description, and Add button.

### 20.1. Plugin Connect Dialog

Clicking Add opens a plugin connection dialog. The dialog content changes to
the selected plugin.

Dialog surfaces:

- Source app and target plugin visual connection.
- Title such as `Connect GitHub`.
- Status: `Approved by your admin`.
- Copy blocks for control, risk, and data shared with plugin.
- Continue action.
- Advanced settings action.

The wireframe does not authenticate, install, or persist plugin connection.

### 20.2. Marketplace Dialog

The marketplace dialog is opened from `+ Add Marketplace`.

Fields:

- Source.
- Git ref.
- Sparse paths.

Actions:

- Cancel.
- Add marketplace.

This is a simulated review form.

## 21. Settings Skills

Route: `#settings-skills`

The Skills screen manages installed skills and per-project availability.

Visible controls:

- Search skills input.
- Skill list rows.
- Scope labels.
- Enabled switches.

Illustrative skill rows:

- Ab Testing
- Ad Creative
- Ads
- Ai Seo
- Analytics
- Aso
- ChrisAI Agents

Clicking a skill opens a skill detail dialog.

Skill detail dialog surfaces:

- Skill icon.
- Skill name.
- Type label `Skill`.
- Skill summary.
- Enabled switch.
- More actions button.
- Detail body split from the selected skill description.
- Uninstall action.
- Try in chat action.

Skill names, descriptions, scopes, and details are placeholders. The wireframe
does not execute skills, uninstall skills, or persist skill availability.

## 22. Settings MCP Servers

Route: `#settings-mcp`

The MCP Servers screen manages external tool/data source connections.

Header:

- Title: MCP Servers.
- Summary: `Connect external tools and data sources.`
- Learn more button.
- Add server action.

Illustrative server rows:

- node_repl
- openaiDeveloperDocs
- stackpress_blog_mcp

Each row has:

- Server name.
- Settings button.
- Enabled switch.

### 22.1. Custom MCP Dialog

Clicking Add server opens `Connect to a custom MCP`.

Top controls:

- Close button.
- Docs link.
- Transport segmented control.

Transport modes:

- STDIO
- Streamable HTTP

STDIO fields:

- Command to launch.
- Arguments.
- Environment variables.
- Environment variable passthrough.
- Working directory.

Streamable HTTP fields:

- URL.
- Bearer token env var.
- Headers.
- Headers from environment variables.

Footer actions:

- Cancel.
- Save.

Server names are placeholders. The wireframe does not connect to or validate
MCP servers.

## 23. Visual System

The r04 visual system is intentionally restrained:

- White paper background.
- Neutral gray rails and borders.
- Black primary text and buttons.
- Rounded corners no larger than functional UI needs.
- System UI typography.
- Monospace only for code and terminal output.

This is not final brand identity. Use it as a behavior and layout reference.

Implementation rules:

- Keep the app dense enough for repeated technical work.
- Avoid oversized landing-page composition.
- Avoid decorative gradients, orbs, or marketing-style hero treatment.
- Use familiar icon buttons for panel, file, provider, model, terminal,
  settings, plugin, skill, and MCP controls.
- Preserve accessible labels for icon-only buttons.

## 24. Interaction Boundaries

The prototype demonstrates interactions but does not own production behavior.

Implemented in prototype:

- Hash route navigation.
- Show More / Show Less message expansion.
- Panel collapse controls.
- Panel resize handles.
- Terminal bottom-panel resize handle.
- Attachment chip rendering after file picker selection.
- Composer popover selection labels.
- Plugin connection dialog.
- Marketplace dialog.
- Skill detail dialog.
- MCP custom server dialog and transport mode toggle.

Simulated or out of scope:

- Real folder open.
- Real clone flow.
- Real workspace file open.
- Prompt submission.
- Provider/model persistence.
- Browser rendering.
- File editing/saving.
- Terminal execution.
- Approval policy enforcement.
- Sandbox configuration persistence.
- Plugin installation/authentication.
- Skill execution/uninstall.
- MCP validation/persistence.

## 25. Accessibility Expectations

The wireframe expresses these accessibility expectations:

- Icon-only buttons need accessible names.
- Focus-visible states should be present.
- Contenteditable composer needs textbox semantics.
- Popovers and modals should receive focus management in production.
- Dialogs should support Escape close.
- Toggles should expose state in production, even if r04 only shows static
  switch styling.
- Text must remain legible and must not overlap controls.

## 26. Implementation Acceptance Notes

Accepted implementation should preserve:

- Folder-first App Start entry.
- Trusted project shell before prompting.
- Three-panel desktop structure.
- Resizable/collapsible side panels.
- Project selection versus active chat selection distinction.
- Browser, Files, Terminal tab order.
- Files explorer and editor state split.
- Messenger-style user/agent ownership.
- Approval boundary surfaced as structured activity.
- Provider/model settings separation.
- Plugins, Skills, and MCP Servers as distinct settings surfaces.

Do not treat r04 placeholder data as production fixtures. Do not hardcode
example project names, session names, file paths, provider names, model names,
plugin names, skill names, MCP server names, branch names, URLs, code snippets,
terminal output, badges, statuses, or sample copy unless a separate product
record explicitly promotes them into requirements, configuration, or final copy.
Use sample data only to understand layout, hierarchy, and state behavior.

When implementing frontend code from this handoff, the expected instruction is:
preserve the r04 behavior and information architecture, but replace illustrative
wireframe data and placeholder visuals with real application state, production
copy, and the accepted design system. If any of those production sources are
missing, leave a clear implementation note or follow-up rather than hardcoding
the r04 examples.

## 27. Required Reconciliation Before Freeze

Before MVP freeze, accepted UI behavior should be reconciled into:

- `.agents/specs/mvp/requirements.md`
- `.agents/specs/mvp/acceptance.md`
- `.agents/specs/mvp/traceability.md`

Reconcile as product behavior, not as prototype implementation detail.

## 28. Open Questions

These are not settled by r04:

- Final product copy.
- Mobile or responsive behavior beyond narrow desktop overflow.
- Production persistence for provider, model, plugin, skill, and MCP settings.
- Exact backend contract for file explorer and editor state.
- Exact terminal execution and approval policy integration.
- Whether Hooks returns to settings navigation after MVP scope changes.
- Final visual branding beyond the neutral MVP shell.
