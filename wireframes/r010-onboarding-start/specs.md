# C4OS Onboarding And Start Wireframe Specification

## Revision Summary

- Revision folder: `wireframes/r010-onboarding-start/`
- Revision: `r010-onboarding-start`
- Status: New revision copied forward in full from `wireframes/r009-settings/`.
- Product area: Provider-dependent application launch, workspace start, desktop AI workspace, and application-level Settings management.
- Requested scope:
  - Preserve the complete r009 SPA, including the existing workspace, Settings views, styles, scripts, workflow links, and supporting files.
  - Add first-provider onboarding and provider-present workspace start as views inside the existing `index.html`, `styles.css`, and `script.js` rather than separate pages.
  - When no provider is configured, launch directly into a standalone Add Provider form with no Settings navigation.
  - After a provider exists, launch into a start screen with Open a folder, Open a workspace, Clone Repository, and exactly three recent workspaces.
  - Continue from provider setup to the start screen and from a workspace selection to the existing main app without reloading the SPA document.
  - Preserve the complete r008 workspace and its existing Chat, Files, Browser, Terminal, and Response Artifact behavior.
  - Add a separate desktop Settings window entered from the native OS application menu; direct-link entry is used for browser review because static HTML cannot render the native menu bar.
  - Cover Providers, Models, Plugins, Skills, MCP Servers, and Configuration as persistent top-level settings destinations.
  - Base the information architecture on current OpenCode, Jan, OpenChamber, and Codex research while preserving the r008 grayscale product language.
  - Make provider connections, model visibility, extension management, server configuration, and raw configuration states meaningfully clickable for review.
  - Preserve the approved r007 layout, artifact interactions, and shared Terminal session behavior while applying small visual refinements before the next design phase.
  - Add a subtle top border to the fixed composer dock so its anchored region is visually separated from the scrollable workspace.
  - End the workspace stage at the composer dock's live top edge rather than underneath it. Measure multiline prompt growth so the stage and its native scrollbar continuously resize with the dock.
  - Remove the composer disclaimer caption and its unused styling, reduce composer dock top padding to 12px, and preserve the shared 760px transcript alignment below the 992px overlay breakpoint so response artifacts do not shift left of assistant responses.
  - Keep Response Artifact headers and footers outside a shared scrolling body so content scrollbars never overlap artifact controls. Remove sticky artifact headers/footers and reuse the same surface/body composition for Browser, File, Folder, and Terminal cards.
  - Treat Reply as a Chat interaction even when the quoted context is a Terminal artifact. Do not execute or create another Terminal artifact from a reply, and label quoted chat references with the first few words of the message instead of its speaker name.
  - Reduce non-Chat composer controls by mode: Files hides Model and Approval while retaining Branch; Browser and Terminal hide Model, Approval, and Branch. Identify every Response Artifact responder as `C4OS` independently of the selected Chat model.
  - Establish reusable Response, Focus, and compact Pane artifact shells. Providers own their bodies and type-specific styles while the shared shells own placement, identity, controls, scrolling boundaries, and context transitions.
  - Make artifact expansion a declared provider capability. A provider may supply substantially different response and focus bodies, while providers without a focus implementation remain inline-only.
  - Stream newly generated agent work and Chat responses with a fast typing effect. Keep seeded transcript history static, respect reduced-motion preferences, and reveal Response Artifact frames only after their work summary completes.
  - Make the Chat paperclip open a native multi-file picker. Show removable draft attachments above the Chat input, preview images, truncate long filenames, allow attachment-only sends, and retain the attachments in the submitted user message.
  - Treat the entire application as a file drop target while Chat mode is active. Show a full-app drop indicator during a file drag and route dropped files into the same attachment draft state as the picker.
  - Remove the right artifact panel, its toggle, separator, tabs, and pane-specific viewers.
  - Expanding a Browser or File artifact replaces the center thread with a full-workspace artifact view.
  - Replace the floating Chat frame with a bottom Chat pane inside the left panel while leaving the composer fixed at the bottom.
  - Show the left Chat pane only while Browser or File is focused; restore Chat to the center from its header control.
  - Make the pane vertically resizable from a draggable top separator, defaulting to 40% and growing to at most 60% of viewport height.
  - Respect the left panel’s collapsed state when an artifact is expanded; expansion must not reopen the panel.
  - Repair left-panel width resizing so repeated drags work, the handle stays aligned, and the center resizes in the same interaction.
  - Replace the fixed 480px cap with a responsive maximum that preserves at least 420px for the center workspace.
  - Add Expand to every Terminal artifact, including completed, interrupted, and running commands.
  - Model one persistent terminal session per chat; all Terminal response artifacts are command snapshots from `shell-1`.
  - Show Stop only for the active long-running foreground process. Stop sends a conceptual `Ctrl+C`, marks that artifact Interrupted, and preserves the shell session.
  - Expanding a Terminal artifact replaces the center thread with that command or process as one continuous terminal body; earlier commands remain available through their transcript artifacts rather than a stacked history view.
  - Keep terminal input inside the body. A process may expose an inline stdin prompt when needed; completion or interruption returns a trailing `$` prompt that creates the next command and response artifact in the same session.
  - Keep the fixed AI prompt composer available and locked to Chat while Terminal is expanded.
  - Add a non-functional Detach Chat control beside Restore Chat as a future native-window affordance.
  - Replace the focused File title/version block with clickable breadcrumbs that open File Explorer.
  - Add fixed line numbers in focused File edit mode, reduce editor padding, and use long sample content to validate scrolling.
  - Allow Files mode to choose a file or folder; sending a folder creates a read-only File Explorer response artifact.
- Explicitly deferred:
  - Native macOS and Windows menu-bar rendering, native secondary-window lifecycle, real credential storage, live provider authentication, remote model discovery, plugin or skill installation, MCP process execution, and configuration writes.
  - Real filesystem access, file writes, browser networking, webpage embedding, shell execution, PTY allocation, process signaling, backend AI responses, and durable persistence.
  - Full-screen interactive terminal programs such as `vim`, `top`, and password-entry flows.
  - Attachment uploading to a backend, persistent attachment storage, real filesystem enumeration, browser sub-tabs, multiple reply targets, credential-specific states, detached native windows, and production security policy.
  - Left-panel content and navigation above the contextual Chat pane.
- Trigger: User request to create a new polish revision and add a top border to `.composer-dock`, dated 2026-07-15.
- Open questions affecting later revisions: None blocking this draft. Minor language and density decisions remain reviewable.

## Source Of Truth

- `wireframes/r007-terminal-session-workspace/`
  - Contributes the complete approved panel layout, artifact-focus behavior, shared Terminal session, response artifacts, and fixed composer.
  - Changed in this revision: The composer dock gains a top divider; no interaction or layout behavior changes.

- `wireframes/r006-file-explorer-artifact/`
  - Contributes the approved File Explorer, focused Browser/File workspace, contextual left Chat pane, shared response containers, prompt actions, and scroll-to-latest control.
  - Changed in this revision: Terminal becomes a focusable artifact backed by one continuous session, with streaming, stdin, Stop, and expanded terminal input.
- User Terminal session request, 2026-07-15.
  - Every Terminal artifact in a chat belongs to the same shell session.
  - Long-running processes expose Stop; every Terminal artifact exposes Expand.
  - Expanded Terminal returns shell input directly inside the terminal body while the fixed AI composer remains Chat.
  - Full-screen interactive programs are deferred; continuous output, ordinary stdin, commands, and `Ctrl+C` are represented.

- `wireframes/r005-left-chat-pane/`
  - Contributes the resizable contextual Chat pane, center artifact focus, fixed composer, direct close controls, and Chat-only artifact-focus mode.
  - Changed in this revision: Files mode gains folder selection and File Explorer artifacts; focused File navigation gains breadcrumbs and a line-numbered editor.
- `wireframes/r004-artifact-focus-swap/`
  - Contributes the center artifact-focus swap, fixed composer, and response artifacts. Its historical Terminal no-Expand rule is superseded by r007.
  - Changed in this revision: The floating Chat frame becomes a bottom pane owned by the left panel.
- User left-pane request and annotated decisions, 2026-07-15.
  - The Chat pane appears only while an artifact is focused.
  - The pane defaults to 40% height, resizes vertically, and may grow to 60% of viewport height.
  - Artifact expansion does not reopen a collapsed left panel.
  - Left-panel resizing must work repeatedly and change center width continuously.

- `wireframes/r003-mode-popover/`
  - Contributes the complete mode-popover composer, unified Response Artifact patterns, inline Browser/File controls, Terminal outputs, and approved manual style refinements.
  - Changed in this revision: The right-pane artifact model is replaced by a center-workspace focus swap and floating thread frame.
- User artifact-focus request and annotated decisions, 2026-07-15.
  - Browser and File artifacts may expand; Terminal artifacts do not.
  - The expanded artifact uses the entire center workspace.
  - Only the thread moves into the floating Chat frame; the composer remains fixed at the bottom.
  - The frame persists its session position and size, defaults to 380px by 420px, and has a 320px by 280px minimum.
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
  - Historical interaction retained as a visual control pattern; the right-pane destination is superseded by the r004 center-workspace viewer.
- User mode request, 2026-07-15.
  - Defines Chat, Files, Browser, and Terminal modes and their input/output behavior.
- User Response Artifact decision, 2026-07-15.
  - Defines structured non-chat outputs as Response Artifacts. The historical r004 Terminal no-Expand decision is superseded by the shared-session Terminal workspace in r007.
- Annotated interaction decisions, 2026-07-15.
  - Modes persist until changed.
  - Files handles one file at a time and receives follow-up instructions through Reply rather than a second field.
  - Browser uses an inline preview and interactive expanded pane.
  - Terminal maintains one continuous shell session.
  - File writes and potentially destructive commands use the existing approval behavior.
  - Historical r002/r003 behavior used right-pane tabs; r004 explicitly replaces tabs with one focused center artifact.
  - Copy uses contextual defaults for each message and artifact type.
  - Reply temporarily replaces the mode selector with a quoted reference, accepts natural language, and restores the previous mode afterward.
  - Browser and File artifacts update in place; Terminal creates new output artifacts in the same session.
  - Artifacts do not focus the workspace automatically.
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

### First Provider Onboarding

- HTML file: `index.html#onboarding`
- Purpose: Gate first launch until one provider is configured.
- Primary user goal: Save the first provider connection and continue into C4OS.
- Layout: Centered blank-window provider card with no Settings navigation.
- Components: C4OS mark, provider type, profile label, compatible endpoint fields, API key with reveal control, Test Connection, and Continue.
- Required states: Provider presets, OpenAI Compatible fields, authentication variants, required-field errors, testing, success, and submitted.
- Navigation in and out: Successful Continue changes the existing SPA hash to `#start`.
- Content: Provider terminology and defaults carried forward from r009 Settings.

### Workspace Start

- HTML file: `index.html#start`
- Purpose: Choose or resume a workspace before entering the main app.
- Primary user goal: Open a folder, open a saved workspace, clone a repository, or resume one of three recent workspaces.
- Layout: Full-window start surface with a compact header, three action cards, and a recent-workspace list.
- Components: C4OS mark, three workspace action cards, exactly three recent rows, and transition status.
- Required states: Default, hover/focus, action-in-progress, and navigation to the main workspace.
- Navigation in and out: All workspace choices transition to `#chat` in the same SPA.

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
- Primary user goal: Create artifacts, reply to messages or artifacts, copy content, and swap Browser/File/Terminal artifacts into the center workspace without losing Chat context.
- Layout: Responsively resizable left panel with a contextual bottom Chat pane, center workspace, and fixed composer. No right panel or floating frame.
- Components: Left-panel toggle and width separator, contextual Chat pane and height separator, messages, thinking activity, mode trigger and popover, mode-specific inputs, prompt controls, response artifact cards, message actions, quoted-reference strip, full-workspace Browser/File viewers, browser navigation, file viewer/editor states, terminal output, approval actions, notifier.
- Required states:
  - Chat, Files, Browser, and Terminal composer modes; mode popover closed and open.
  - Chat with multiple draft attachments, an image thumbnail, a truncated long filename, a removed attachment, the full-app drag target, and attachments retained in a submitted user message.
  - Reply targeting for an ordinary message and each artifact type.
  - Browser/File artifact default and updated states.
  - Terminal session with completed, running, and interrupted command cards; one selected-process expanded body; returned shell input and contextual process stdin.
  - Chat centered, Browser focused, File focused, Terminal focused, left Chat pane vertically resized, panel collapsed while focused, artifact-to-artifact focus swap, and Chat restored.
  - Hover/focus message actions, copy success, file approval required, file saved, and browser navigation history change.
- Navigation in and out: Direct workflow links; no secondary screen.
- Content: Illustrative chat, browser, file, and terminal activity sufficient to exercise all requested patterns.

## Workflow Starting Points

### First Launch Without A Provider

- Starting screen link: `./index.html#onboarding`
- Intended user role or mode: New user without a configured provider.
- Happy path: Select a provider, enter required credentials, test the connection, continue to `#start`, then open a workspace and enter `#chat`.
- Alternate paths: Switch to OpenAI Compatible, change authentication mode, reveal the API key, or submit incomplete fields.

### Launch With A Provider

- Starting screen link: `./index.html#start`
- Intended user role or mode: Returning or already configured user.
- Happy path: Open one of three recent workspaces and enter the existing Chat workspace.
- Alternate paths: Open a folder, open a saved workspace, clone a repository, or open provider Settings.

### Continue A Chat

- Starting screen link: `./index.html#chat`
- Intended user role or mode: Chat.
- Happy path: Enter a natural-language prompt and receive a normal assistant response.
- Alternate paths: Copy or reply to either message.

### Open And Revise A File

- Starting screen link: `./index.html#files`
- Intended user role or mode: Files.
- Happy path: Browse or enter one file path, open a File artifact, reply with a natural-language change request, approve the proposed edit, and inspect the updated version.
- Alternate paths: Cancel reply, reject edits, copy file contents, focus the File artifact in the workspace, and restore Chat.

### Open And Operate A Webpage

- Starting screen link: `./index.html#browser`
- Intended user role or mode: Browser.
- Happy path: Enter an address, create a Browser artifact, use inline navigation, reply with a natural-language request, and focus the updated artifact across the center workspace.
- Alternate paths: Copy URL, refresh, navigate back/forward, resize the left Chat pane vertically, focus a File artifact from the pane, collapse/reopen the left panel without changing focus, and restore Chat.

### Run And Question Terminal Output

- Starting screen link: `./index.html#terminal`
- Intended user role or mode: Terminal.
- Happy path: Expand a running Terminal artifact, inspect its streaming process body, interrupt it with Stop, and run another command from the `$` prompt returned directly beneath `^C`.
- Alternate paths: Expand a completed command, copy or reply to an output card, close the terminal workspace, and restore the previously selected composer mode.

## Layout System

### Artifact Focus Workspace

- Used by: `index.html`.
- Regions: Left panel with contextual bottom Chat pane, center workspace stage, normal thread host, full-workspace artifact host, toolbar, and fixed composer.
- Desktop behavior:
  - Left panel begins open and resizes from 180px to the smaller of 55% of the workspace or the width that preserves a 420px center.
  - Repeated width drags continuously update both the left panel and center boundary.
  - Browser/File/Terminal Expand replaces the center thread with a full-workspace viewer.
  - The actual thread element moves into a bottom Chat pane inside the left panel; the fixed composer does not move.
  - The pane defaults to 40% viewport height, has a 220px minimum where space permits, and grows to at most 60%.
  - The active artifact bubble becomes a compact selected placeholder inside the left Chat pane.
  - Expanding another artifact from the pane swaps the center viewer and restores the previous bubble.
  - Restoring Chat from the pane header returns the thread to its normal center host and restores the active artifact bubble.
  - Focusing an artifact never changes the left panel’s open/collapsed state.
- Narrow behavior:
  - Below 860px, the left panel becomes an overlay.
  - The same bottom Chat pane remains inside the left-panel overlay when the panel is open.
  - Composer controls wrap without horizontal document overflow.
- Shared files: `styles.css`, `script.js`.

### Mode-Aware Composer

- Used by: `index.html`.
- Regions: Optional quoted-reference strip, mode-specific input, shared lower toolbar with mode trigger, send/open/run action.
- Behavior: The mode trigger opens an upward popover. Its active label and icon reflect the persisted mode. In Chat mode it appears immediately left of the paperclip. Reply temporarily hides the mode trigger and uses Chat-style natural-language input; submit or cancel restores the trigger and previous mode.
- Shared files: `styles.css`, `script.js`.

### Contextual Left Chat Pane

- Used by: `index.html` while a Browser or File artifact is focused.
- Regions: Draggable top height separator, compact header, thread viewport, and Chat restore control.
- Behavior: Contains only the existing thread and appears at the bottom of the left panel. Its height persists across focus swaps for the current session. It follows—but never changes—the left panel’s current open/collapsed state.
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

- Chat: Toolbar-free CodeMirror Markdown editor. Canonical source markers remain visible while headings, emphasis, code, links, quotes, and list prefixes receive lightweight syntax effects; Enter sends and Shift+Enter inserts or continues a Markdown line.
- Files: Single file path field plus Browse control and Open action.
- Browser: Address field plus Open action.
- Terminal: Monospace command field with prompt glyph plus Run action.
- Reply: The same CodeMirror Markdown editor regardless of target artifact type.
- Accessibility: Mode-specific labels, placeholders, keyboard submission, and visible focus.
- Files: `styles.css`, human-maintained `markdown-source.js`, generated `markdown.js`, and `script.js`.

### Message Actions

- Appears: User messages, assistant messages, and artifacts on hover or focus-within.
- Variants and states: Copy and Reply appear beneath message and artifact bubbles on the lower left; Browser/File/Terminal Expand remains in the artifact header; running Terminal artifacts additionally expose Stop.
- Inputs and outputs: Copy writes the contextual default; Reply activates quoted context; Expand focuses the artifact in the center workspace; Terminal Stop interrupts only the active foreground process.
- Accessibility: Real buttons remain keyboard focusable even when visually subdued; accessible names include the target identity.
- Files: `styles.css`, `script.js`.

### Agent Response Streaming

- Appears: Newly generated assistant messages and Response Artifacts in either the center transcript or compact left Chat pane.
- Sequence: Open the work disclosure, type `Thinking…`, stream the concise work summary, type `Working…` plus its activity, collapse to the final elapsed label, and then reveal the responder and final output.
- Chat response: The final bubble remains hidden until work completes, then safe Markdown progressively renders with a visible cursor and receives one clean final render.
- Response Artifact: The C4OS identity, provider frame, and contextual actions remain hidden until work completes, then reveal together without typing provider-owned body content.
- History: Seeded and previously completed transcript entries never replay their animation on page load.
- Accessibility: Active responses expose `aria-busy="true"`; the attribute is removed after completion. Reduced-motion preferences skip typing and transitions while preserving the same final state.
- Files: `styles.css`, `markdown.js`, `script.js`.

### Response Artifact Card

- Appears: In the center thread for Browser, File, and Terminal output.
- Shared shell: `artifact-shell artifact-shell--response artifact-provider artifact-provider--<type>` owns transcript placement, the C4OS preamble, provider frame, and lower-left Copy/Reply actions. The provider supplies the framed header/body/footer content rather than recreating the outer response markup.
- Shared frame: `artifact-frame` owns the bordered response container; `artifact-frame__header` and `artifact-frame__footer` stay static while `artifact-frame__body` is the only scrolling region.
- Shared anatomy: Full-width thinking-activity disclosure, artifact-type icon and C4OS identity row, constrained left-aligned provider frame, and a lower-left Copy/Reply action row. Expand appears only when the provider registry declares the artifact expandable.
- Browser variant: One consolidated toolbar header with back, forward, refresh, read-only current address, and far-right Expand; the browser type icon remains in the model row; page preview and navigation-history indicator follow.
- File variant: Path, file type/version, read-only text preview, inline Edit/Cancel/Save controls, line-numbered scrollable and vertically resizable editor, proposed-change state, approval actions, saved state.
- Terminal variant: Executed command as the header title, session/working directory, status/exit result, and output-only body. A running card streams in place and shows Stop; completed or interrupted cards are immutable snapshots.
- Extension contract: A future provider registers its type and capabilities, supplies provider-specific response body markup, and adds an `artifact-provider--<type>` style hook. Unknown providers default to a generic icon and non-expandable behavior.
- Accessibility: Article landmark, descriptive label, named controls, readable output, and status text.
- Files: `styles.css`, `script.js`.

### Full-Workspace Artifact Viewer

- Appears: In the center stage after Browser, File, Folder, or Terminal Expand.
- Shared shell: `artifact-shell artifact-shell--focus artifact-provider artifact-provider--<type>` owns the full-workspace region and shared controls. Providers may supply a focus body that differs substantially from their compact response body.
- Variants and states: Browser, File, Folder, and Terminal; active; swaps directly when another artifact is expanded.
- Inputs and outputs: Mirrors and updates the focused inline artifact state. The left Chat pane restore control returns the viewer to its bubble.
- Accessibility: Named region, preserved control labels, visible focus, and a clearly named Chat restore action.
- Files: `styles.css`, `script.js`.

### Compact Pane Artifact Shell

- Appears: On existing Response Artifacts after the real thread moves into the contextual left Chat pane.
- Shared shell: The original response markup gains `artifact-shell--pane` and `data-artifact-context="pane"`; it is not rebuilt as a separate provider view.
- Behavior: The modifier removes response-width constraints, adapts provider controls to the narrow pane, and preserves the artifact's state, Copy/Reply actions, and expansion capability.
- Restoration: Returning Chat to the center removes the modifier and restores `data-artifact-context="response"` on every artifact.
- Accessibility: The same article, controls, labels, and focus order remain available in both transcript contexts.
- Files: `styles.css`, `script.js`.

### Expanded Terminal Session

- Appears: In the full center workspace after expanding any Terminal artifact.
- Variants and states: Completed command, running foreground process, contextual stdin-ready, interrupted, and command-ready.
- Inputs and outputs: Shows the selected command and its output as the main terminal body without history cards or a separate command bar. Stop interrupts that foreground process without closing the shell. Completion or interruption appends an inline `$` prompt; submitting it creates the next command/output artifact in `shell-1` and makes that command the active expanded body.
- Accessibility: Labeled terminal region, named Stop/Close controls, status text, selected-command state, labeled input, and keyboard submission.
- Files: `styles.css`, `script.js`.

### Copy Notifier

- Appears: Temporarily above the composer after Copy.
- Variants and states: Hidden and visible success.
- Inputs and outputs: Reports what was copied without changing transcript state.
- Accessibility: Polite live region.
- Files: `styles.css`, `script.js`.

## Interaction And State Contract

### Route The Application Launch

- Trigger: Initial document load or hash change.
- Affected elements: Launch SPA, onboarding view, start view, workspace shell, and Settings shell.
- Before: Provider configuration state is unknown or the URL selects a review route.
- After: Exactly one top-level SPA surface is visible.
- Visible result: No provider shows onboarding; a configured provider shows the start screen; workspace and Settings hashes show their existing r009 surfaces.
- Navigation result: Empty hash resolves to `#onboarding` unless the prototype provider flag exists, then resolves to `#start`.
- Responsible module: Launch router in `script.js`.

### Configure The First Provider

- Trigger: Change provider/authentication fields, reveal the API key, test, or submit.
- Affected elements: Onboarding provider form and status region.
- Before: No provider configured.
- After: Valid submission stores the prototype provider flag and routes to `#start`.
- Visible result: Compatible-only fields and required validation respond to the selected provider and authentication method.
- Responsible module: Onboarding provider controller in `script.js`.

### Enter A Workspace

- Trigger: Activate a primary workspace action or recent-workspace row.
- Affected elements: Start status and top-level SPA route.
- Before: Start screen visible.
- After: A short simulated opening status appears, then the existing workspace opens at `#chat`.
- Responsible module: Workspace start controller in `script.js`.

### Run A Terminal Command From The Composer

- Trigger: Submit a value from the Terminal-mode composer.
- Affected elements: Conversation thread, Terminal response artifacts, and shared terminal-session record.
- Before: Zero or more earlier Terminal artifacts may exist, including a running or interrupted process.
- After: The submitted command is represented by a new user command message and a new Terminal response artifact.
- Visible result: Earlier Terminal artifacts remain immutable; the new command and its output appear in their own response container.
- Navigation result: Remains on `#terminal`.
- Responsible module: Terminal-mode submit controller and `createTerminalArtifact()` in `script.js`.
- Process input boundary: Input intended for an already running process remains owned by the expanded Terminal session’s inline prompt; the global Terminal composer always creates a new command response.

### Preserve File Edit State Across Focus

- Trigger: Expand an inline File artifact while it is being edited, or close a focused File artifact while it is being edited.
- Affected elements: Inline File response, focused File workspace, editable content, line numbers, and Edit/Cancel/Save controls.
- Before: The file has one shared editing flag and draft value.
- After: Focus changes presentation context without changing editing mode or discarding the draft.
- Visible result: Expanding an editing file opens the focused editor immediately; closing it restores the inline editor with the same unsaved text and editing controls.
- Navigation result: The URL mode remains unchanged while focus opens or closes.
- Responsible module: Shared File artifact state, `renderFocusViewer()`, `restoreChat()`, and file-editor input synchronization in `script.js`.

### Restore A Focused File Explorer To Its Inline State

- Trigger: Activate X while a File Explorer artifact is focused at a folder or opened child file.
- Affected elements: Focused File Explorer state and its inline File Explorer response artifact.
- Before: The focused Explorer may have navigated to another folder and may show a child file in view or edit mode.
- After: Focus closes and Chat returns while the inline File Explorer response adopts the focused Explorer’s current folder, opened file, view/edit state, and unsaved draft.
- Visible result: The response artifact shows the same opened child file or folder state that was visible immediately before X.
- Navigation result: Focus closes normally and the prior composer mode is restored.
- Responsible module: Explorer state synchronization in `restoreChat()`, `updateFolderCard()`, and Explorer editor input handling.

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
- After: A new Terminal artifact card is appended and the shared session history grows. Completed commands freeze immediately; recognized long-running commands remain active and stream until stopped.
- Visible result: Command, output, status or exit result, shared working directory, Expand, and conditionally Stop appear.
- Responsible module: Terminal session controller in `script.js`.

### Expand, Operate, And Stop A Terminal Session

- Trigger: Activate Expand on any Terminal artifact; submit the expanded terminal input; activate Stop while a foreground process is running.
- Before: The thread is centered and `shell-1` contains one or more command entries.
- After expanding: The selected command or process becomes the uninterrupted terminal body in the center workspace and the conversation moves into the left Chat pane. The fixed AI composer is locked to Chat.
- After terminal input: A process-specific stdin prompt appears inline only when the process requests input. When the shell is idle, the returned `$` prompt creates a new command and matching response artifact in the same conversation/session, then replaces the expanded body with that command.
- After Stop: The active entry receives `^C`, becomes Interrupted with exit 130, and the shell prompt becomes command-ready without losing history or working directory.
- Responsible module: Terminal session and focus-swap controllers in `script.js`.

### Focus An Artifact And Restore Chat

- Trigger: Activate Expand on a Browser/File/Folder/Terminal artifact; activate Restore Chat in the left Chat pane header to restore.
- Before: Chat owns the center workspace and the artifact is inline.
- After focusing: The selected artifact owns the center workspace, the existing thread moves into the left panel’s bottom Chat pane, the composer remains fixed, and the source bubble becomes a compact selected placeholder.
- Focused controls: The composer is temporarily locked to Chat. Inline Browser navigation and inline File editing are read-only while they sit in the contextual Chat pane. The full-workspace Browser keeps navigation, the File keeps Edit, and the Terminal keeps session input/Stop; all include a direct Close control.
- After restoring: The thread returns to its normal center host, the pane hides without losing height, the full-workspace viewer clears, and the artifact returns to its original bubble.
- Alternate focus: Expanding another Browser/File artifact from the Chat pane swaps the center viewer and source placeholder without creating tabs. A collapsed left panel stays collapsed.
- Responsible module: Focus-swap and left Chat pane controllers in `script.js`.

### Resize The Left Chat Pane

- Trigger: Drag/focus the horizontal separator at the top of the Chat pane.
- Before: Pane at its remembered height or 40% default.
- After: Pane height is clamped between its minimum and 60% of viewport height and retained for later artifact focus sessions.
- Responsible module: Chat pane height controller in `script.js`.

### Message Action Visibility

- Trigger: Pointer hover or keyboard focus within a message/artifact.
- Before: Actions visually hidden but keyboard-accessible.
- After: Actions become visible.
- Visible result: Compact action row appears without shifting message layout.
- Responsible module: CSS hover/focus-within rules.

### Resize Or Toggle The Left Panel

- Trigger: Drag/focus a separator or activate a panel toggle.
- State and visible result: Width updates continuously across repeated drags; the separator remains aligned; the center boundary moves with the panel; collapse and reopen preserve width and any active Chat pane height.
- Responsible module: Panel layout controller in `script.js`.

## Artifact Provider Extension Contract

- Registry: Add the provider type to `artifactProviders` with explicit capabilities such as `expandable`.
- Response: Build the provider-owned frame body, then pass it through `createResponseArtifactNode()` and `responseArtifactMarkup()` so the preamble, attributes, actions, and pane context remain shared.
- Focus: When `expandable` is true, return the provider's full-workspace body through `focusArtifactMarkup()`.
- Pane: Do not build another provider body. `setThreadArtifactContext()` applies or removes `artifact-shell--pane` on the existing response node.
- Styling: Put provider-only rules below `artifact-provider--<type>`. Use `artifact-shell--response`, `artifact-shell--focus`, or `artifact-shell--pane` only when the provider must differ by context.
- Fallback: An unregistered provider is non-expandable and receives the generic artifact icon; it does not break the shared response shell.

## Library Plan

- `styles.css`: Shared `artifact-shell`, `artifact-frame`, and `artifact-provider` contracts plus provider-specific Browser, File, Folder, and Terminal body styles.
- `script.js`: Provider capability registry, shared response/focus/pane shell builders, and the existing one-session Terminal state model.
- `index.html`: Seeded examples use the same response-shell contract as dynamically created artifacts.
- `workflows.html`: Copied from `r006`; the Terminal workflow entry opens the revised terminal state.
- Lucide-guided inline SVG icons: panel, chat, file, globe, terminal, copy, reply, expand, close, navigation, refresh, save, approve, reject, and status.
- State persistence: Existing workspace interactions remain in-memory; URL hashes select SPA views and modes; one localStorage flag simulates whether the first provider has been configured.

Prototype behavior intentionally omits real file, network, browser, terminal, and AI operations while preserving visible states and core workflows.

## Page Build Plan

### `workflows.html`

- Title: AI Workspace Workflows.
- Layout: Centered workflow launcher.
- Components: Onboarding, start-screen, workspace-mode, and Settings workflow links with descriptions.
- Imports: `./styles.css`.
- Initial state: All launch, workspace, and Settings workflows available.
- Interaction hooks: Standard relative links into routes owned by the single `index.html` SPA.

### `index.html`

- Title: AI Artifact Workspace.
- Layout: Left panel plus center focus workspace and mode-aware fixed composer.
- Components: All components listed above.
- Imports: `./styles.css`, generated `./markdown.js`, and `./script.js`.
- Initial state:
  - Empty hash routes to first-provider onboarding unless the prototype provider flag is already present, then routes to the workspace start screen.
  - `#onboarding` and `#start` provide deterministic review entry points independently of the stored flag.
  - Left panel open, Chat centered, no right panel.
  - Mode selected from hash or Chat by default.
  - Thread contains realistic Chat, Browser, File, and a running Terminal example backed by earlier session history.
  - No reply target or focused artifact; contextual left Chat pane hidden at remembered/default height.
- Interaction hooks: `data-panel-*`, `data-mode-*`, `data-reply-*`, `data-copy-*`, `data-artifact-*`, `data-browser-*`, `data-file-*`, `data-terminal-*`, `data-focus-*`, `data-left-chat-pane-*`, and existing prompt-control hooks.

## Functional Acceptance Checks

- `specs.md`, `notes.md`, `workflows.html`, `index.html`, `styles.css`, `markdown-source.js`, generated `markdown.js`, and `script.js` exist and match this contract.
- The full r009 revision file set is copied into r010 before onboarding changes are applied.
- First-provider onboarding and workspace start are views inside `index.html`; their styles and behavior live in `styles.css` and `script.js`.
- No-provider launch shows the Add Provider form without Settings navigation.
- Valid provider setup routes to the workspace start screen inside the same SPA.
- Provider-present launch shows Open a folder, Open a workspace, Clone Repository, and exactly three recent workspaces.
- Every workspace start action routes to the retained r009 main app without replacing the document.
- Workflow launcher links to all four mode starting points.
- The compact mode control is immediately left of the paperclip in Chat mode and opens an upward popover.
- The popover presents Chat, Files, Browser, and Terminal with a clear selected state.
- Chat, Files, Browser, and Terminal modes visibly and functionally change the composer input and active trigger label/icon.
- Chat and Reply use one toolbar-free CodeMirror source editor, Enter to send, Shift+Enter for Markdown-aware line continuation, Cmd/Ctrl+B and Cmd/Ctrl+I formatting shortcuts, and native undo history.
- Selecting editor text and pasting a URL creates a Markdown link; pasted raw web addresses are highlighted and render as links after submission.
- User and assistant Chat bubbles render safe Markdown while Copy and Reply retain the Markdown source; thinking and working details remain plain text.
- Chat draft and submitted attachments render newest first with their original one-based reference numbers and show metadata as `<EXT> · <size>`.
- Submitted attachment groups remain visually separate from the ordinary user prompt bubble.
- Selected mode persists after submission.
- Files accepts one file or folder path/selection; a file creates a File artifact and a folder creates a File Explorer artifact.
- Browser accepts an address, creates a Browser artifact, and supports inline back, forward, refresh, and address navigation updates.
- Browser artifact navigation and read-only current address share one header bar, with Expand aligned at the far right.
- Terminal accepts commands, creates response artifacts, and maintains one continuous session.
- Terminal headers show the command while the output body does not repeat it.
- Every Terminal artifact exposes Expand; only the active running artifact exposes Stop.
- Expanding any Terminal artifact shows that selected command or process as the main body; prior session commands remain represented by their inline artifacts.
- The expanded Terminal returns an inline `$` command prompt after completion or interruption and may surface stdin inline when a foreground process requests it.
- Commands submitted inside the expanded Terminal append matching response artifacts to the conversation.
- Stop interrupts only the foreground process, changes it to Interrupted/exit 130, and leaves the shell session ready.
- File artifacts enter a line-numbered, scrollable, vertically resizable inline edit mode and return to the default bubble height after Cancel or Save.
- Files, Browser, and Terminal composer inputs use consistent boxed leading icons; Files Browse is icon-only.
- User, assistant, and artifact Copy/Reply actions appear on hover/focus and work.
- Reply hides the mode trigger, shows a removable quoted reference, uses natural-language input, and restores the previous mode and trigger after cancel or submit.
- Browser and File reply workflows update the original artifact card in place.
- Terminal reply workflow appends a new output card without mutating the target card.
- File proposed edits support Approve-to-save and Reject states.
- Browser/File/Folder/Terminal Expand replaces the center thread with a full-workspace viewer and shows the thread-only Chat pane at the bottom of the left panel.
- The composer remains fixed at the bottom and fully usable while an artifact is focused.
- Artifact focus temporarily locks the composer to Chat, hides the mode chooser, and restores the previously selected mode after Chat is restored.
- Inline Browser artifacts in the contextual Chat pane keep only their read-only address and Expand control; inline File artifacts remain read-only without Edit, Save, Cancel, or approval controls.
- The focused Browser viewer includes a direct Close control beside its read-only address, including when the left panel is collapsed.
- The focused File viewer fills the available center height above the composer, uses a pencil icon for Edit, and includes a direct Close control.
- Focused File headers use clickable folder breadcrumbs instead of repeated filename/version metadata.
- Focused File edit mode displays fixed left-side line numbers, keeps them synchronized while scrolling, and uses reduced editor padding.
- File Explorer artifacts show read-only breadcrumb navigation and folder/file rows; folder rows open that folder in the center workspace.
- File Explorer folder rows replace the current listing with path-specific child folders/files in both inline and focused views.
- File rows replace the current listing with a read-only file surface in both inline and focused views; parent breadcrumbs return to File Explorer.
- The Chat pane header includes a non-functional Detach control immediately left of Restore Chat; native-window behavior is deferred.
- The Chat pane defaults to 40% height, resizes vertically to at most 60%, and remembers its session height.
- Expanding another Browser/File artifact from the pane swaps the focused viewer and restores the prior bubble.
- Restore Chat returns the centered thread and the focused artifact’s original bubble.
- Artifact expansion preserves the left panel’s current open/collapsed state.
- Left-panel width resizing supports repeated drags, keeps its separator aligned, and continuously resizes the center between a 180px panel minimum and responsive maximum.
- Completed, running, interrupted, seeded, and newly generated Terminal artifacts all have Expand.
- No right panel, right resize handle, right toggle, artifact tabs, or pane-close controls remain.
- Contextual Copy values and success notifier work.
- Panel resizing, collapse, fixed composer, thread scrolling, and thinking-activity disclosure continue to work.
- Wide and narrow layouts remain readable without horizontal document overflow.
- Rendered UI contains no annotations, TODOs, review notes, or implementation commentary.
- All links and imports are document-relative and require no build tooling.
- This revision represents the first draft of the left Chat pane layout. Review feedback may refine pane height, left-panel width, or restore behavior before phase approval.

## R009 Settings SPA Extension Contract

### Revision Summary

- Revision folder: `wireframes/r009-settings/`.
- Revision: r009, updated in place to correct the implementation architecture.
- Product area: C4OS desktop workspace and application settings.
- Requested scope: Preserve the existing workspace and add Providers, Models, Runtimes, Configuration, Plugins, Skills, MCP Servers, and Advanced Policies as linkable states of the same static single-page application.
- Architecture constraint: `index.html`, `styles.css`, and `script.js` are the only application HTML/CSS/JS files. Settings and Advanced Policies must not create parallel page or asset bundles.
- Explicitly deferred: Native OS application-menu chrome remains simulated; production persistence, provider calls, model discovery, installations, external-editor launch, and MCP connections are not implemented.

### Source Of Truth Additions

- `wireframes/r008-workspace-polish/`: supplies the existing workspace shell, grayscale tokens, compact typography, borders, controls, and interaction tone.
- User settings request, navigation peg, provider/model/runtime/plugin/skill/MCP pegs, and browser annotations from 2026-07-17: define destination order, copy, layout, forms, states, and approved refinements.
- Advanced Policies scope attachment from 2026-07-17: defines the nine authority groups, 71 identities, descriptions, and four policy values.
- OpenCode, Jan, OpenChamber, and official OpenAI Codex sources reviewed for the earlier settings rounds: contribute reference patterns only; the r008/r009 artifact remains the visual authority.
- User architecture correction from 2026-07-17: settings HTML, CSS, and JavaScript belong in the existing SPA files.

### Screen Inventory Addition

#### C4OS Workspace SPA

- HTML file: `index.html`.
- Purpose: Host the conversation workspace, all settings destinations, and Advanced Policies in one static document.
- Primary user goals: Work in chat/artifact modes, open Settings, configure resources, and return to the same workspace without a document navigation.
- Layouts:
  - Workspace shell: existing resizable left panel, centered thread, composer, artifact focus states, and dropzone.
  - Settings shell: viewport-filling 224px navigation plus independently scrolling destination content.
  - Policy browser: Settings shell content with search, a 190px group rail, and a scrollable policy list.
- Required linkable states:
  - `#chat`, `#files`, `#browser`, and `#terminal` for workspace composer modes.
  - `#settings/providers`, `#settings/models`, `#settings/runtimes`, `#settings/configuration`, `#settings/plugins`, `#settings/skills`, and `#settings/mcp`.
  - `#settings/advanced-policies`, with Configuration selected in the settings navigation.
- Navigation:
  - The workspace header exposes a Settings control for browser review; the product source remains the native OS application menu.
  - Back to C4OS changes the hash to `#chat` and restores the existing workspace DOM and state.
  - Settings navigation changes only the active SPA destination and hash.
  - Configuration’s Advanced link opens the Advanced Policies SPA state.

### Workflow Starting Point Additions

- Connect a provider and select models: `./index.html#settings/providers`.
- Choose a runtime: `./index.html#settings/runtimes`.
- Manage plugins and marketplaces: `./index.html#settings/plugins`.
- Manage installed skills: `./index.html#settings/skills`.
- Configure an MCP server: `./index.html#settings/mcp`.
- Customize configuration: `./index.html#settings/configuration`.
- Customize per-tool policies: `./index.html#settings/advanced-policies`.
- Every path stays in `index.html`; Back to C4OS returns to `./index.html#chat`.

### Layout System Addition

#### Settings SPA Layout

- Used by: all `#settings/*` states in `index.html`.
- Regions: Back to C4OS, grouped settings navigation, active destination content, dialog backdrop, and notifier.
- Desktop behavior: Settings replaces the visible workspace shell without unloading it; navigation remains fixed while content scrolls.
- Narrow behavior: below 680px the navigation compresses to icons and content retains the desktop-first minimum review width.
- Shared files: `styles.css` and `script.js`.

#### Policy Browser Layout

- Used by: `#settings/advanced-policies`.
- Regions: page header and Save Policies, search, authority-group rail, active/search result header, and policy rows.
- Narrow behavior: below 780px the group rail stacks above results and scrolls horizontally.
- Shared files: `styles.css` and `script.js`.

### Component Inventory Addition

- Settings navigation: Back to C4OS, Settings label, seven icon-and-label buttons, divider, selected state, and hash routing.
- Page headers: destination title, supporting copy, and optional aligned action.
- Provider/model/runtime/skill/MCP resource rows with compact identity, aligned action, badge, radio, or switch columns.
- Switches: buttons with `role="switch"`, `aria-checked`, keyboard activation, and visible on/off state.
- Dialogs: shared modal backdrop and close behavior for Providers, MCP, Marketplace, Plugin details, and Skill details.
- Provider form: type-dependent presets, OpenAI-compatible URL/auth/header fields, secret reveal, validation, test, and save.
- Plugin directory: left-aligned Installed/Directory tabs, search, marketplace chooser, Add Marketplace dialog, and state-aware Install/Uninstall details.
- Skills: searchable list with circular document icons, synchronized availability, details, Uninstall, and Try in chat.
- MCP form: STDIO and Streamable HTTP panels with repeatable fields.
- Configuration: approval policy, Advanced link, workspace restore, shell environment, browser environment, and external config action.
- Advanced Policies: group rail, cross-group search, fixed trailing selects, dirty rows, Save Policies, and empty results.
- Notifier: transient simulated feedback shared by all settings states.

### Interaction And State Contract Addition

- SPA routing:
  - A `hashchange` router toggles visibility between the existing workspace shell and settings shell.
  - Supported `#settings/<slug>` routes activate one settings page; invalid settings slugs fall back to Providers.
  - Advanced Policies keeps Configuration selected while replacing the content state.
  - The workspace DOM is hidden, not rebuilt, so in-memory workspace state survives a settings visit.
- Providers:
  - Add/Edit opens one type-dependent dialog.
  - OpenRouter, Hugging Face, and OpenAI use preset endpoints plus API keys.
  - OpenAI Compatible exposes Label, API base URL, API key, Auth (Bearer token, API key header, None), optional header name, and JSON Headers.
  - LiteLLM is represented as OpenAI Compatible.
  - Test and Save validate visible fields; labels must be unique; switches affect provider-derived model availability.
- Models:
  - Search and provider filtering define visible results.
  - Models default enabled; row switches update availability.
  - The bulk action disables visible results, or reads Enable results when every visible result is disabled.
  - Refresh provides simulated progress and completion feedback.
- Runtimes:
  - OpenCode and Pi are mutually exclusive draft choices.
  - Save Runtime enables only when the draft differs from the saved selection.
- Plugins:
  - Installed/Directory tabs replace panels.
  - Directory search and marketplace chooser filter discovery.
  - Details include a logo placeholder, capabilities, Website, Terms, Privacy Policy, and installed-state-aware Install or Uninstall.
  - Add Marketplace accepts Source, Git ref, and sparse paths.
- Skills:
  - Search filters installed skills.
  - Row and dialog switches share one in-memory availability value.
  - Uninstall removes the skill; Try in chat returns simulated feedback.
- MCP:
  - Add and Configure use the same dialog.
  - STDIO exposes command, arguments, environment variables, passthrough, and working directory.
  - Streamable HTTP exposes URL, bearer-token environment variable, headers, and environment-backed headers.
  - Repeaters add/remove rows; Save validates transport-specific required fields.
- Configuration:
  - Controls update simulated in-memory state.
  - Browser Environment offers All browsers, Per project, Per chat session, and None.
  - Advanced changes the route to `#settings/advanced-policies`.
  - Open config.toml externally reports a simulated action.
- Advanced Policies:
  - Authority-group selection replaces rows.
  - Search spans all nine groups.
  - Draft changes enable Save Policies; reverting all changes disables it; save promotes the draft and notifies.
- Dialogs close through explicit controls, backdrop, or Escape.

### Library Plan Addition

- `index.html`: adapted to include the Settings shell, destination markup, Advanced Policies state, dialogs, notifier, and browser-review Settings entry.
- `styles.css`: adapted to include settings and policy styles scoped under the Settings SPA wrapper so workspace component styles do not drift.
- `script.js`: adapted to include settings interactions, policy rendering, and top-level SPA routing while retaining the existing workspace runtime.
- `markdown.js`: retained for the existing workspace editor runtime.
- Icons remain inline Lucide-guided grayscale SVGs.
- No settings-specific HTML/CSS/JS files are part of this revision.

### Page Build Plan Addition

#### `index.html`

- Title: AI Artifact Workspace.
- Layouts: Workspace shell, Settings SPA Layout, and Policy Browser Layout.
- Imports: `./styles.css`, `./markdown.js`, and `./script.js`.
- Initial state: route-derived; absent or unsupported hashes show Chat, while supported settings hashes show the matching destination.
- Interaction hooks: existing workspace hooks plus `data-settings-*`, `data-provider-*`, `data-model-*`, `data-runtime-*`, `data-plugin-*`, `data-skill-*`, `data-mcp-*`, `data-config-*`, `data-policy-*`, `data-dialog-*`, and `data-notifier`.
- Links: all settings workflow entry points are hashes within this document.

### Functional Acceptance Checks

- `index.html`, `styles.css`, and `script.js` contain the complete workspace and settings implementation.
- `settings.html`, `settings.css`, `settings.js`, `advanced-policies.html`, `advanced-policies.css`, and `advanced-policies.js` do not exist.
- Every workflow link targets `index.html`.
- Workspace and every settings destination are reachable by direct hash without reloading another document.
- Back to C4OS restores the workspace; revisiting settings preserves in-memory workspace state.
- Settings CSS is scoped so generic settings controls do not override workspace controls.
- Providers, Models, Runtimes, Plugins, Skills, MCP Servers, Configuration, and Advanced Policies retain their approved content and interactions.
- Dialogs close through explicit controls, backdrop, and Escape.
- Wide and narrow desktop layouts remain readable without horizontal document overflow.
- The rendered UI contains no annotations, TODOs, review notes, or implementation commentary.
- All imports are document-relative and the artifact requires no build tooling.
