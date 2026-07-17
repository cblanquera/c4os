# Mode Popover Wireframe Specification

## Revision Summary

- Revision folder: `wireframes/r003-mode-popover/`
- Revision: `r003-mode-popover`
- Status: New major revision copied forward from `wireframes/r002-response-artifact-modes/specs.md`.
- Product area: Desktop AI chat composer mode discovery and selection.
- Requested scope:
  - Replace the persistent mode row with a compact active-mode control in the lower composer toolbar.
  - Position the mode control immediately to the left of the paperclip attachment control in Chat mode.
  - Open an upward popover containing Chat, Files, Browser, and Terminal choices.
  - Preserve the selected mode, mode-specific inputs, Response Artifacts, replies, and right-pane tabs from `r002`.
- Explicitly deferred:
  - Real filesystem access, file writes, browser networking, webpage embedding, shell execution, backend AI responses, and durable persistence.
  - Multiple-file selection, directories, browser sub-tabs, multiple reply targets, credential-specific states, and production security policy.
  - Left-panel content and navigation.
- Trigger: User request for a new mode-popover revision dated 2026-07-15.
- Open questions affecting later revisions: None blocking this draft. Minor language and density decisions remain reviewable.

## Source Of Truth

- `wireframes/r002-response-artifact-modes/`
  - Contributes the complete artifact workspace, mode-specific composer behavior, Response Artifacts, contextual replies, right-pane tabs, and shared response-container styling.
  - Changed in this revision: Only the composer mode-selection presentation moves from a persistent row to a toolbar popover.
- User mode-popover request, 2026-07-15.
  - Defines a mode popover placed to the left of the paperclip attachment icon.
- Browser annotations, Round 2, 2026-07-15.
  - Browser artifacts consolidate their type icon, back, forward, refresh, read-only current address, and Expand control into one header bar.
  - File artifacts support inline Edit, Cancel, and Save controls. Edit mode adds line numbers plus a scrollable, vertically resizable editor; returning to read mode restores the default artifact height.
  - Terminal artifact headers use the executed command as the title and the output body omits the repeated command line.
  - Files mode uses an icon-only Browse control; Browser and Terminal input prefixes use the same boxed icon treatment.
- Browser annotations, Round 3, 2026-07-15.
  - Browser, File, and Terminal type icons move from their artifact headers into the assistant identity row beside the active model label.
  - Artifact headers no longer repeat their type icons.
  - Terminal composer layout reserves the full width of its boxed `$` prefix so it does not collide with the command input.
- Browser annotation, Round 4, 2026-07-15.
  - The expanded right-pane Browser viewer uses a dedicated horizontal navigation header with Back, Forward, Refresh, and address controls.
  - User-authored style refinements present before this round remain preserved; the fix is scoped to the right-pane Browser header.
- User mode request, 2026-07-15.
  - Defines Chat, Files, Browser, and Terminal modes and their input/output behavior.
- User Response Artifact decision, 2026-07-15.
  - Defines structured non-chat outputs as Response Artifacts with an expand control that opens the right pane.
- Annotated interaction decisions, 2026-07-15.
  - Modes persist until changed.
  - Files handles one file at a time and receives follow-up instructions through Reply rather than a second field.
  - Browser uses an inline preview and interactive expanded pane.
  - Terminal maintains one continuous shell session.
  - File writes and potentially destructive commands use the existing approval behavior.
  - The right pane uses artifact tabs; closing a tab closes only its view.
  - Copy uses contextual defaults for each message and artifact type.
  - Reply temporarily replaces the mode selector with a quoted reference, accepts natural language, and restores the previous mode afterward.
  - Browser and File artifacts update in place; Terminal creates new output artifacts in the same session.
  - Artifacts do not open the right pane automatically.
  - Copy and Reply actions appear on hover or keyboard focus.
- Browser annotations, Round 2, 2026-07-15.
  - The thinking-activity disclosure appears above the model label and response bubble.
  - Response Artifact type icons are vertically centered against the title and metadata block.
  - Artifact Copy and Reply actions appear beneath the artifact on the lower left, matching ordinary message actions; Expand remains in the artifact header.
- Browser annotations, Round 3, 2026-07-15.
  - Every assistant response begins with a full-width thinking-activity disclosure above the assistant icon and model label.
  - Response Artifacts use the same assistant-response preamble: thinking activity, then assistant icon and model label, then the artifact surface.
  - Artifact type icons are centered inside their fixed square containers on both axes independently of title alignment.
- Browser annotations, Round 4, 2026-07-15.
  - Ordinary assistant response bubbles begin on a new row beneath the icon/model identity and align flush with the thinking disclosure and icon.
  - Response Artifacts use the same constrained left-aligned reading width as assistant response bubbles instead of stretching across the full thread.
- Final CSS refinement, Round 5, 2026-07-15.
  - Assistant bubbles and inline Response Artifact surfaces share one response-container radius contract, including the compact 5px lower-left corner.
  - Repeated response geometry, model-row alignment, and contextual-action rules are consolidated into shared CSS selectors and tokens.

## Screen Inventory

### Workflow Launcher

- HTML file: `workflows.html`
- Purpose: Provide review entry points into the mode and artifact workflows.
- Primary user goal: Open the AI workspace at a useful workflow state.
- Layout: Centered workflow launcher with four links.
- Components: Product title, workflow links, short descriptions.
- Required states: Default, hover, and focus.
- Navigation in and out: Links to `./index.html` with hashes for Chat, Files, Browser, and Terminal review starting points.
- Content: Realistic workflow labels without internal review language.

### AI Chat Workspace

- HTML file: `index.html`
- Purpose: Review one SPA that supports conversational chat plus persistent Files, Browser, and Terminal artifacts.
- Primary user goal: Create artifacts, reply to messages or artifacts, copy content, and open artifacts in right-pane tabs.
- Layout: Resizable three-panel shell with blank left panel, center thread/composer, and contextual right artifact workspace.
- Components: Panel toggles, resize separators, messages, thinking activity, mode trigger and popover, mode-specific inputs, prompt controls, response artifact cards, message actions, quoted-reference strip, artifact tabs, artifact viewer, browser navigation, file viewer/editor states, terminal output, approval actions, notifier.
- Required states:
  - Chat, Files, Browser, and Terminal composer modes; mode popover closed and open.
  - Reply targeting for an ordinary message and each artifact type.
  - Browser/File artifact default and updated states.
  - Terminal session with multiple immutable command/output cards.
  - Right panel closed, open with one tab, open with multiple tabs, active-tab changes, and tab closure.
  - Hover/focus message actions, copy success, file approval required, file saved, and browser navigation history change.
- Navigation in and out: Direct workflow links; no secondary screen.
- Content: Illustrative chat, browser, file, and terminal activity sufficient to exercise all requested patterns.

## Workflow Starting Points

### Continue A Chat

- Starting screen link: `./index.html#chat`
- Intended user role or mode: Chat.
- Happy path: Enter a natural-language prompt and receive a normal assistant response.
- Alternate paths: Copy or reply to either message.

### Open And Revise A File

- Starting screen link: `./index.html#files`
- Intended user role or mode: Files.
- Happy path: Browse or enter one file path, open a File artifact, reply with a natural-language change request, approve the proposed edit, and inspect the updated version.
- Alternate paths: Cancel reply, reject edits, copy file contents, expand to right-pane file tab, close and reopen the tab.

### Open And Operate A Webpage

- Starting screen link: `./index.html#browser`
- Intended user role or mode: Browser.
- Happy path: Enter an address, create a Browser artifact, use inline navigation, reply with a natural-language request, and expand the updated artifact into the right pane.
- Alternate paths: Copy URL, refresh, navigate back/forward, switch between right-pane tabs.

### Run And Question Terminal Output

- Starting screen link: `./index.html#terminal`
- Intended user role or mode: Terminal.
- Happy path: Run a command, create an output artifact, reply to that output in natural language, and append a new command/output artifact in the same terminal session.
- Alternate paths: Copy command/output, expand the persistent terminal session, close and reopen the terminal tab.

## Layout System

### Three-Panel Artifact Workspace

- Used by: `index.html`.
- Regions: Blank left panel, center chat workspace, contextual right artifact workspace, toolbar, scrollable thread, fixed composer.
- Desktop behavior:
  - Left panel begins open at 228px.
  - Right panel begins closed and opens at 380px when an artifact is expanded.
  - Both panels remain independently resizable between 180px and 480px.
  - Expanding an artifact opens or focuses its right-pane tab without leaving the thread.
- Narrow behavior:
  - Below 860px, side panels become overlays.
  - Artifact expansion opens the right overlay and keeps the center state intact.
  - Composer controls wrap without horizontal document overflow.
- Shared files: `styles.css`, `script.js`.

### Mode-Aware Composer

- Used by: `index.html`.
- Regions: Optional quoted-reference strip, mode-specific input, shared lower toolbar with mode trigger, send/open/run action.
- Behavior: The mode trigger opens an upward popover. Its active label and icon reflect the persisted mode. In Chat mode it appears immediately left of the paperclip. Reply temporarily hides the mode trigger and uses Chat-style natural-language input; submit or cancel restores the trigger and previous mode.
- Shared files: `styles.css`, `script.js`.

### Right-Pane Artifact Tabs

- Used by: Right panel in `index.html`.
- Regions: Tab strip, close-pane control, active artifact content.
- Behavior: Multiple artifact tabs may remain open. Expanding an already-open artifact focuses its existing tab. Closing a tab does not destroy the artifact or its session.
- Shared files: `styles.css`, `script.js`.

## Component Inventory

### Composer Mode Trigger And Popover

- Appears: At the start of the lower composer toolbar when no reply target is active; immediately left of the paperclip in Chat mode.
- Variants and states: Popover closed/open; Chat, Files, Browser, and Terminal selected; hover and focus.
- Inputs and outputs: Trigger activation opens the popover. Choosing an option changes the mode-specific input and primary action, updates the trigger label/icon, persists the selected mode, and closes the popover.
- Accessibility: The trigger exposes `aria-haspopup` and `aria-expanded`; the popover uses menu semantics with a checked current mode, supports Escape, and returns focus predictably.
- Files: `styles.css`, `script.js`.

### Quoted Reply Strip

- Appears: Above the mode-specific input while replying; the mode trigger is temporarily hidden.
- Variants and states: Ordinary message, Browser, File, Terminal artifact.
- Inputs and outputs: Shows icon, source label, excerpt/title, and remove button. Remove restores the prior mode without submitting.
- Accessibility: Labeled region and explicit remove control.
- Files: `styles.css`, `script.js`.

### Mode-Specific Input

- Chat: Multiline natural-language textarea.
- Files: Single file path field plus Browse control and Open action.
- Browser: Address field plus Open action.
- Terminal: Monospace command field with prompt glyph plus Run action.
- Reply: Natural-language textarea regardless of target artifact type.
- Accessibility: Mode-specific labels, placeholders, keyboard submission, and visible focus.
- Files: `styles.css`, `script.js`.

### Message Actions

- Appears: User messages, assistant messages, and artifacts on hover or focus-within.
- Variants and states: Copy and Reply appear beneath message and artifact bubbles on the lower left; artifact-only Expand remains in the artifact header.
- Inputs and outputs: Copy writes the contextual default; Reply activates quoted context; Expand opens/focuses an artifact tab.
- Accessibility: Real buttons remain keyboard focusable even when visually subdued; accessible names include the target identity.
- Files: `styles.css`, `script.js`.

### Response Artifact Card

- Appears: In the center thread for Browser, File, and Terminal output.
- Shared anatomy: Full-width thinking-activity disclosure, artifact-type icon and model label row, constrained left-aligned bordered artifact surface with text-only identity where applicable, title and status/meta, header-level Expand control, compact preview, and a lower-left Copy/Reply action row beneath the card.
- Browser variant: One consolidated toolbar header with boxed browser icon, back, forward, refresh, read-only current address, and far-right Expand; page preview and navigation-history indicator.
- File variant: Path, file type/version, read-only text preview, inline Edit/Cancel/Save controls, line-numbered scrollable and vertically resizable editor, proposed-change state, approval actions, saved state.
- Terminal variant: Executed command as the header title, session/working directory, exit status, and output-only body; cards are immutable after creation.
- Accessibility: Article landmark, descriptive label, named controls, readable output, and status text.
- Files: `styles.css`, `script.js`.

### Artifact Tab And Viewer

- Appears: Right panel after artifact expansion.
- Variants and states: Browser, File, Terminal; active/inactive; closeable.
- Inputs and outputs: Select focuses a viewer; close removes only the pane tab; Expand from thread reopens it.
- Accessibility: Tablist/tab/tabpanel semantics, selected state, and named close controls.
- Files: `styles.css`, `script.js`.

### Copy Notifier

- Appears: Temporarily above the composer after Copy.
- Variants and states: Hidden and visible success.
- Inputs and outputs: Reports what was copied without changing transcript state.
- Accessibility: Polite live region.
- Files: `styles.css`, `script.js`.

## Interaction And State Contract

### Change Composer Mode

- Trigger: Open the toolbar mode popover and activate Chat, Files, Browser, or Terminal.
- Affected element: Mode trigger, popover, input surface, placeholder, primary action label/icon, and relevant supporting controls.
- Before: One persisted mode.
- After: Selected mode persists in memory.
- Visible result: Composer input and active trigger label/icon change immediately; the popover closes and the thread remains unchanged.
- Navigation result: Hash updates to the selected mode for review linking.
- Responsible module: Composer mode controller in `script.js`.

### Start And Cancel A Reply

- Trigger: Activate Reply on any message or artifact; activate `×` to cancel.
- Affected element: Mode trigger, quoted-reference strip, and composer input.
- Before: Persisted mode trigger is visible.
- After: The mode trigger is hidden, quoted context appears, and the input becomes natural-language Chat; cancel restores the prior mode and trigger.
- Visible result: Target icon, label, and excerpt/title appear above the input.
- Navigation result: None.
- Responsible module: Reply controller in `script.js`.

### Submit A Reply

- Trigger: Send a natural-language reply while quoted context is active.
- Affected element: Thread, target artifact when applicable, reply strip, and composer mode.
- Before: Reply target bound.
- After:
  - Ordinary message: Append user and assistant messages.
  - Browser: Append the user instruction, update the original Browser artifact in place, and add a navigation/activity entry.
  - File: Append the user instruction and update the original File artifact to proposed changes requiring approval.
  - Terminal: Append the user instruction and a new Terminal output artifact in the same session without changing the earlier card.
- Visible result: Updated or appended content; prior persisted mode returns.
- Navigation result: None.
- Responsible module: Reply submission controller in `script.js`.

### Copy Content

- Trigger: Activate Copy.
- Contextual value:
  - User or assistant message: Message text.
  - Browser artifact: Current URL.
  - File artifact: Current visible file contents.
  - Terminal artifact: Command plus output.
- Visible result: Success notifier appears briefly.
- Navigation result: None.
- Responsible module: Copy controller in `script.js` with clipboard API and safe fallback.

### Create A File Artifact

- Trigger: Select Files mode, choose or enter one file, and activate Open.
- Before: No new artifact.
- After: One File artifact is appended; selected mode remains Files.
- Visible result: Read-only file preview with Copy, Reply, and Expand actions.
- Responsible module: Artifact factory in `script.js`.

### Propose And Approve File Edits

- Trigger: Reply to File artifact; activate Approve or Reject in the updated artifact.
- Before: File artifact version is read-only.
- After: Reply creates proposed changes. Approve saves immediately, increments the illustrative version, updates original card, and records history. Reject restores read-only unchanged state.
- Visible result: Diff preview and approval actions become saved status or disappear.
- Responsible module: File artifact controller in `script.js`.

### Create And Navigate A Browser Artifact

- Trigger: Select Browser mode, enter address, and activate Open; use back, forward, refresh, or address controls.
- Before: No artifact or current page state.
- After: Browser artifact is appended or updated in place with a navigation-history entry.
- Visible result: The read-only header address, page preview content, and navigation count update.
- Responsible module: Browser artifact controller in `script.js`.

### Edit A File Artifact Inline

- Trigger: Activate Edit in a File artifact header; activate Cancel or Save while editing.
- Before: Compact read-mode preview at the default response-artifact height.
- After: Edit mode exposes a line-numbered textarea that scrolls and can be resized vertically. Cancel restores the unchanged compact preview. Save persists the edited text, increments the illustrative version, and restores the compact preview.
- Visible result: Header actions switch between Edit and Cancel/Save; footer status reflects Editing or Saved.
- Responsible module: Inline File editor controller in `script.js`.

### Create Terminal Output

- Trigger: Select Terminal mode, enter command, and activate Run.
- Before: Persistent terminal session has zero or more commands.
- After: A new immutable Terminal artifact card is appended; session history grows.
- Visible result: Command, output, exit status, and shared working directory appear.
- Responsible module: Terminal session controller in `script.js`.

### Open, Focus, And Close Artifact Tabs

- Trigger: Activate Expand on artifact; select or close a right-pane tab.
- Before: Right pane closed or showing another artifact.
- After: Target tab opens/focuses. Closing removes only the view; artifact data remains in the thread and can be reopened.
- Visible result: Right pane opens, tab strip updates, active viewer changes.
- Responsible module: Artifact tab controller plus panel layout controller in `script.js`.

### Message Action Visibility

- Trigger: Pointer hover or keyboard focus within a message/artifact.
- Before: Actions visually hidden but keyboard-accessible.
- After: Actions become visible.
- Visible result: Compact action row appears without shifting message layout.
- Responsible module: CSS hover/focus-within rules.

### Resize Or Toggle Panels

- Trigger: Drag/focus a separator or activate a panel toggle.
- State and visible result: Same accessible resizing and collapse behavior as `r001`; artifact expansion programmatically opens the right panel.
- Responsible module: Panel layout controller in `script.js`.

## Library Plan

- `styles.css`: Copied from `r002` and adapted. Replaces the persistent mode row with a compact toolbar trigger and upward popover while retaining the shared response, artifact, and responsive contracts.
- `script.js`: Copied from `r002` and adapted. Replaces tab-style mode activation with an accessible popover controller while retaining all artifact and reply behavior.
- `index.html`: Copied from `r002` and adapted with the toolbar mode trigger and popover.
- `workflows.html`: Copied from `r002`; the same four workflow entry points initialize the active popover mode.
- Lucide-guided inline SVG icons: panel, chat, file, globe, terminal, copy, reply, expand, close, navigation, refresh, save, approve, reject, and status.
- State persistence: In-memory for the review session; URL hash selects an initial mode. No localStorage is required.

Prototype behavior intentionally omits real file, network, browser, terminal, and AI operations while preserving visible states and core workflows.

## Page Build Plan

### `workflows.html`

- Title: AI Workspace Workflows.
- Layout: Centered workflow launcher.
- Components: Four workflow links with icons and descriptions.
- Imports: `./styles.css`.
- Initial state: All four workflows available.
- Interaction hooks: Standard relative links to `./index.html#chat`, `#files`, `#browser`, and `#terminal`.

### `index.html`

- Title: AI Artifact Workspace.
- Layout: Three-panel artifact workspace and mode-aware composer.
- Components: All components listed above.
- Imports: `./styles.css`, `./script.js`.
- Initial state:
  - Left panel open, right panel closed.
  - Mode selected from hash or Chat by default.
  - Thread contains realistic Chat, Browser, File, and Terminal examples.
  - No reply target active and no artifact tabs open.
- Interaction hooks: `data-panel-*`, `data-mode-*`, `data-reply-*`, `data-copy-*`, `data-artifact-*`, `data-browser-*`, `data-file-*`, `data-terminal-*`, `data-tab-*`, and existing prompt-control hooks.

## Functional Acceptance Checks

- `specs.md`, `notes.md`, `workflows.html`, `index.html`, `styles.css`, and `script.js` exist and match this contract.
- Workflow launcher links to all four mode starting points.
- The compact mode control is immediately left of the paperclip in Chat mode and opens an upward popover.
- The popover presents Chat, Files, Browser, and Terminal with a clear selected state.
- Chat, Files, Browser, and Terminal modes visibly and functionally change the composer input and active trigger label/icon.
- Selected mode persists after submission.
- Files accepts one file path/selection and creates a File artifact.
- Browser accepts an address, creates a Browser artifact, and supports inline back, forward, refresh, and address navigation updates.
- Browser artifact navigation and read-only current address share one header bar, with Expand aligned at the far right.
- Terminal accepts commands, creates immutable output cards, and maintains one continuous session.
- Terminal headers show the command while the output body does not repeat it.
- File artifacts enter a line-numbered, scrollable, vertically resizable inline edit mode and return to the default bubble height after Cancel or Save.
- Files, Browser, and Terminal composer inputs use consistent boxed leading icons; Files Browse is icon-only.
- User, assistant, and artifact Copy/Reply actions appear on hover/focus and work.
- Reply hides the mode trigger, shows a removable quoted reference, uses natural-language input, and restores the previous mode and trigger after cancel or submit.
- Browser and File reply workflows update the original artifact card in place.
- Terminal reply workflow appends a new output card without mutating the target card.
- File proposed edits support Approve-to-save and Reject states.
- Artifact Expand opens/focuses a right-pane tab without automatic opening on creation.
- Multiple artifact tabs remain open; closing one closes only its view and it can be reopened from the transcript.
- Contextual Copy values and success notifier work.
- Panel resizing, collapse, fixed composer, thread scrolling, and thinking-activity disclosure continue to work.
- Wide and narrow layouts remain readable without horizontal document overflow.
- Rendered UI contains no annotations, TODOs, review notes, or implementation commentary.
- All links and imports are document-relative and require no build tooling.
- This revision represents the complete first draft of the mode-popover refinement. Review feedback may refine popover density, placement, labels, or keyboard behavior before phase approval.
