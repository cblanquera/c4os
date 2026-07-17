# AI Chat Shell Wireframe Specification

## Revision Summary

- Revision folder: `wireframes/r001-ai-chat-shell/`
- Revision: `r001-ai-chat-shell`
- Status: Existing revision updated for review round 3.
- Product area: Desktop AI chat shell.
- Requested scope: A single-page desktop layout with resizable left and right panels, a center chat thread, and a prompt control fixed to the bottom of the center region.
- Included prompt options in toolbar order: Attach, model, approval mode, Git branch, microphone, and send.
- Explicitly deferred: Content and navigation inside the left and right panels; production AI responses; real file upload, speech capture, repository access, model switching, and approval-policy enforcement.
- Trigger: User request dated 2026-07-15.
- Open questions affecting later revisions: What product content belongs in each side panel; whether either panel should start collapsed; exact terminology and available choices for approval, branch, and model controls.

## Source Of Truth

- User request, 2026-07-15.
  - Contributes the three-panel layout, resizable side panels, central AI chat, fixed prompt control, required prompt options, and blank side-panel scope.
  - Gap: No existing brand, app, or content model was supplied, so this revision uses neutral illustrative chat content and grayscale styling.
- Browser annotations, review round 2, 2026-07-15.
  - Remove visible `You` labels from user prompts.
  - Replace assistant `AI` labels with the selected model name, shown as `GPT-5` in this revision.
  - Present assistant responses in left-aligned chat bubbles.
  - Add an expandable `Worked for 1 min 5 sec` disclosure above each assistant bubble.
  - Reorder the prompt toolbar to Attach, model, approval, and Git branch.
  - Safety interpretation: The expanded disclosure contains a concise, user-facing work summary and does not expose private chain-of-thought reasoning.
- Reference image and direct feedback, review round 3, 2026-07-15.
  - Replace the boxed, bordered work-summary panel with an unboxed stream of short progress paragraphs and muted activity rows.
  - Remove the visible `Work summary` heading and list treatment.
  - Preserve the expandable `Worked for <duration>` control.
  - Safety interpretation: The stream represents user-facing progress and tool activity, not private chain-of-thought reasoning.
- `chrisai-designing` bundled resizable panel layout.
  - Contributes the accessible separator pattern, desktop resize behavior, and narrow-screen overlay behavior.
  - Adaptation: The layout and interaction logic are flattened into revision-local `styles.css` and `script.js` for this single-screen SPA.

## Screen Inventory

### Workflow Launcher

- HTML file: `workflows.html`
- Purpose: Provide the required review entry point into the wireframe workflow.
- Primary user goal: Open the AI chat workspace.
- Layout: Centered launch card.
- Components: Product title, short description, primary link.
- Required states: Default and link focus/hover.
- Navigation in and out: Links to `./index.html`; no return route required.
- Content: Neutral AI chat workspace label.

### AI Chat Workspace

- HTML file: `index.html`
- Purpose: Review the foundational desktop AI chat shell.
- Primary user goal: Read a conversation and compose a prompt while controlling chat context options.
- Layout: Resizable three-panel shell with a fixed center header, scrollable thread, and prompt dock.
- Components: Panel toggle buttons, accessible resize separators, user chat bubbles, assistant chat bubbles, assistant model labels, expandable thinking-activity streams, prompt textarea, attach button, model selector, approval selector, branch selector, microphone toggle, send button, and compact option menus.
- Required states: Both side panels open, either panel collapsed, resizing, assistant thinking activity collapsed or expanded, selector menu open, microphone active, file attached, empty prompt, populated prompt, and submitted message.
- Navigation in and out: Direct entry from `workflows.html`; no secondary screen.
- Content: Illustrative user and AI exchange about outlining a desktop chat workspace.

## Workflow Starting Points

- Workflow name: Continue an AI chat.
- Starting screen link: `./index.html`
- Intended user role or mode: Desktop AI assistant user.
- Happy path: Review the thread, enter a prompt, optionally change prompt options, and submit.
- Alternate paths included: Resize or collapse either side panel, attach a local file name for preview, choose approval/branch/model values, and toggle microphone state.

`workflows.html` exposes this workflow as the single review entry point.

## Layout System

### Three-Panel AI Workspace

- Used by: `index.html`.
- Regions: Blank left panel, center workspace, blank right panel, center toolbar, scrollable message thread, and bottom prompt dock.
- Desktop behavior: Left and right panels begin open at 248px and 288px. Pointer dragging or keyboard arrows resize them between 180px and 420px. Panel toggles collapse and restore each panel.
- Narrow behavior: Below 860px, side panels become overlay surfaces, start collapsed, and remain independently toggleable. The center fills the viewport.
- Shared files: `styles.css` and `script.js`.

### Workflow Launcher

- Used by: `workflows.html`.
- Regions: Full-height neutral page and centered start card.
- Responsive behavior: Card width contracts with the viewport.
- Shared file: `styles.css`.

## Component Inventory

### Panel Toggle Button

- Appears: Center toolbar, one control for each panel.
- Variants and states: Left/right; expanded/collapsed; hover, focus, and pressed.
- Input/output: Click or keyboard activation changes the associated panel visibility.
- Accessibility: Text alternative through `aria-label`, `aria-controls`, and synchronized `aria-expanded`.
- Files: `styles.css`, `script.js`.

### Resize Separator

- Appears: Between the center region and each side panel.
- Variants and states: Left/right; idle, hover, focus, and active drag.
- Input/output: Pointer position or arrow/Home/End keys change the associated panel width.
- Accessibility: Focusable `role="separator"` with orientation and synchronized min/max/current values.
- Files: `styles.css`, `script.js`.

### Chat Message

- Appears: Center thread.
- Variants and states: Right-aligned user bubble without a visible speaker label; left-aligned assistant bubble with model label and collapsed or expanded thinking-activity stream.
- Input/output: Submitted prompt text creates a user message and an illustrative AI response.
- Accessibility: Sequential message structure; assistant identity is visibly labeled by model name; native disclosure controls expose expanded state.
- Files: `styles.css`, `script.js`.

### Assistant Thinking Activity

- Appears: Above each assistant response bubble.
- Variants and states: Collapsed by default and expanded.
- Input/output: Activating the `Worked for <duration>` summary reveals an unboxed sequence of short progress paragraphs and muted tool-activity rows.
- Accessibility: Native `details` and `summary` behavior provides keyboard activation and expanded-state semantics.
- Visual treatment: No container box, background, border, heading, or bulleted summary list around the expanded content.
- Content boundary: Shows user-facing progress and tool activity suitable for the user; private chain-of-thought reasoning is not represented.
- Files: `styles.css`, `script.js`.

### Prompt Composer

- Appears: Fixed to the bottom of the center region.
- Variants and states: Empty, focused, populated, attachment present, microphone active, and submitting.
- Input/output: Textarea receives a prompt; Enter submits; Shift+Enter inserts a newline.
- Accessibility: Explicit textarea label, keyboard submission, visible focus, and live status updates.
- Files: `styles.css`, `script.js`.

### Prompt Option Button And Menu

- Appears: Prompt toolbar after Attach, ordered model, approval, then branch.
- Variants and states: Closed/open; current choice; hover, focus, and selected menu item.
- Input/output: Button opens a compact menu; choosing an item updates the button label and closes the menu.
- Accessibility: `aria-haspopup`, synchronized `aria-expanded`, menu semantics, Escape dismissal, and focus return.
- Files: `styles.css`, `script.js`.

### Attach Button

- Appears: Prompt toolbar.
- Variants and states: Idle and attached-file preview.
- Input/output: Activates a hidden file input; selected file name appears as a removable chip.
- Accessibility: Button label and removable attachment control.
- Files: `styles.css`, `script.js`.

### Microphone Toggle

- Appears: Prompt toolbar.
- Variants and states: Inactive and listening.
- Input/output: Toggles an illustrative listening state; no audio is captured.
- Accessibility: Synchronized `aria-pressed` and live status text.
- Files: `styles.css`, `script.js`.

## Interaction And State Contract

### Resize A Side Panel

- Trigger: Drag its separator, or focus the separator and press ArrowLeft, ArrowRight, Home, or End.
- Affected element: Left or right panel width.
- Before: Current panel width within the allowed range.
- After: Width clamped between 180px and 420px.
- Visible result: Center workspace grows or contracts; separator position follows the panel edge.
- Navigation result: None.
- Responsible module: Panel layout section in `script.js`.

### Toggle A Side Panel

- Trigger: Activate the matching toolbar control.
- Affected element: Side panel, center offsets, separator, and toggle state.
- Before: Open or closed.
- After: Opposite visibility state.
- Visible result: Panel slides out or in and center workspace expands or contracts.
- Navigation result: None.
- Responsible module: Panel layout section in `script.js`.

### Change A Prompt Option

- Trigger: Open approval, branch, or model control and select an item.
- Affected element: Option menu and the control's visible value.
- Before: Current value and closed menu.
- After: Selected value and closed menu.
- Visible result: Control text updates.
- Navigation result: None.
- Responsible module: Prompt options section in `script.js`.

### Expand Assistant Thinking Activity

- Trigger: Activate the `Worked for <duration>` disclosure above an assistant bubble.
- Affected element: The selected assistant message's thinking-activity stream.
- Before: Collapsed summary showing duration only.
- After: Expanded stream showing concise progress paragraphs and activity rows.
- Visible result: Unboxed content appears between the duration control and assistant bubble.
- Navigation result: None.
- Responsible module: Native `details` behavior; no JavaScript module required.

### Attach A File

- Trigger: Activate Attach and select one local file.
- Affected element: Hidden file input and attachment chip.
- Before: No attachment.
- After: Selected filename stored in memory for the current page session.
- Visible result: Removable filename chip appears above the textarea.
- Navigation result: None.
- Responsible module: Attachment section in `script.js`.

### Toggle Microphone

- Trigger: Activate the microphone button.
- Affected element: Microphone button and prompt status.
- Before: Inactive or listening.
- After: Opposite state.
- Visible result: Pressed treatment and live status text update.
- Navigation result: None.
- Responsible module: Microphone section in `script.js`.

### Submit A Prompt

- Trigger: Activate Send or press Enter without Shift while the textarea contains text.
- Affected element: Message list, textarea, attachment chip, and scroll position.
- Before: Populated prompt.
- After: New user message and illustrative AI reply; prompt and attachment clear.
- Visible result: New conversation content appears and scrolls into view.
- Navigation result: None.
- Responsible module: Conversation section in `script.js`.

### Dismiss Menus

- Trigger: Escape, outside click, or opening a different prompt option.
- Affected element: Any open prompt option menu.
- Before: One menu open.
- After: All menus closed; Escape returns focus to the originating control.
- Visible result: Menu disappears.
- Navigation result: None.
- Responsible module: Prompt options section in `script.js`.

## Library Plan

- Bundled `base/tokens.css`, `base/reset.css`, and `base/base.css`: Adapted and flattened into `styles.css`; only tokens and primitives needed by this revision are retained.
- Bundled `layouts/panel-layout-resizable.html`, `layouts/panel-layout.css`, and `layouts/panel-layout.js`: Adapted into the `index.html` shell plus revision-local CSS and JS.
- Lucide icon guidance: Used to create the inline grayscale panel, paperclip, shield, branch, model, microphone, remove, and send SVGs in `index.html`.
- Prompt composer, left/right chat bubble patterns, assistant thinking-activity disclosures, and compact prompt option menus: Created for this revision in `index.html`, `styles.css`, and `script.js`.
- State persistence: In-memory only; no localStorage or cross-page state is needed.

Prototype-only behavior intentionally omits real file transfer, microphone capture, repository discovery, approval enforcement, model calls, and persistence.

## Page Build Plan

### `workflows.html`

- Title: AI Chat Wireframe Workflows.
- Layout: Centered workflow launcher.
- Components: Heading, summary, link button.
- Imports: `./styles.css`.
- Initial state: One available workflow.
- Interaction hooks: Standard link to `./index.html`.
- Links: `./index.html`.

### `index.html`

- Title: AI Chat Workspace.
- Layout: Three-panel AI workspace.
- Components: Panel toggles, resize separators, user and assistant chat bubbles, model labels, expandable thinking-activity streams, prompt composer, option menus, attach control, microphone toggle, and send control.
- Imports: `./styles.css` and `./script.js`.
- Initial state: Both panels open on wide desktop, closed on narrow viewport; default options are `Ask`, `main`, and `GPT-5`.
- Interaction hooks: `data-panel-*`, `data-menu-*`, `data-option-*`, `data-attach-*`, `data-mic`, and `data-prompt-*` attributes.
- Links: None.

## Functional Acceptance Checks

- `specs.md` matches `workflows.html`, `index.html`, `styles.css`, and `script.js`.
- `workflows.html` links to the AI chat workflow start.
- The single requested SPA screen exists.
- Both side panels are visibly blank and independently resizable.
- Panel separators work with pointer and keyboard input.
- The center thread scrolls independently while its prompt composer remains fixed at the bottom.
- User prompt bubbles do not show a visible `You` label.
- Assistant responses show the model name `GPT-5`, use left-aligned chat bubbles, and include working expandable thinking activity without a surrounding box, border, heading, or list treatment.
- Attach, model, approval, branch, microphone, and send controls are present in that order and are interactive.
- Enter submits and Shift+Enter inserts a newline.
- Menus dismiss through selection, Escape, and outside click.
- The interface remains readable at wide and narrow viewport sizes.
- Rendered UI contains no annotations, TODOs, or implementation notes.
- All links and imports are document-relative.
- The artifact opens as static HTML and requires no build system.
- This revision represents the complete requested first-round shell scope; phase approval may advance to defining side-panel content or another explicitly chosen wireframe area.
