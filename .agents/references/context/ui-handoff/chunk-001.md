# UI Handoff - Chunk 001

Parent: `index.md`
Focus: Sections 1-9: usage, normative versus illustrative content, source material, product frame, artifact contract, global layout, App Start, New Session, and prompt composer behavior.

## C4OS UI Handoff Specification

Status: draft handoff
Updated: 2026-06-20
Primary artifact: `wireframes/r04-single-page-app/index.html`

This document explains the r04 C4OS wireframes as an implementation handoff.
It is written for an agent or engineer who has not opened the r04 folder. The
goal is to remove guesswork about layout, state, interactions, simulated
behavior, and implementation boundaries.

### 1. How To Use This Document

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

### 2. Normative Versus Illustrative Content

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
  Models, Runtimes, Configuration, Plugins, Skills, and MCP Servers.

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

### 3. Source Material

The handoff is derived from:

- `wireframes/r04-single-page-app/`
- `wireframes/screens.md`
- `plans/product-interface.md`
- `plans/pegs/*.png`
- `.agents/specs/research/requirements.md`
- `.agents/specs/research/acceptance.md`

The r04 prototype is the latest review target. Earlier revisions remain useful
as design history, but r04 is the current handoff baseline.

### 4. Product Frame

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

### 5. r04 Artifact Contract

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
| `#settings-runtimes` | Settings Runtimes |
| `#settings-configuration` | Settings Configuration |
| `#settings-plugins` | Settings Plugins |
| `#settings-skills` | Settings Skills |
| `#settings-mcp` | Settings MCP Servers |

Unknown or stale routes fall back to App Start.

### 6. Global Layout Model

C4OS uses a desktop shell with three functional regions:

1. Left navigation panel.
2. Center workbench.
3. Right workspace tools panel.

This is not a responsive card layout. Do not place the app shell inside a
decorative outer card. Do not stack the shell into mobile sections unless a
future responsive design explicitly approves that behavior. At narrow desktop
review widths, preserving the desktop shell with horizontal overflow is
acceptable.

#### 6.1. Left Navigation Panel

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

#### 6.2. Center Workbench

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

#### 6.3. Right Workspace Tools Panel

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

#### 6.4. Resizing And Collapse

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

### 7. App Start Screen

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

### 8. New Session Screen

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

### 9. Prompt Composer

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

#### 9.1. Composer Control Row

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

#### 9.2. Context Strip

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

#### 9.3. Attachments

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
