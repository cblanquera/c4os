# Workspace Shell And Conversation Contract

## Shell Regions

The workspace has a resizable left project panel, a center stage, a 58px title header, a scrollable transcript or focused artifact host, and a fixed bottom composer dock. There is no right panel, right resize handle, artifact tab strip, or automatic artifact pane.

The title header contains only the active or pending thread title. Do not restore the removed “Chat, files, browser, and terminal” subtitle.

## Project And Session Navigation

Order the left panel as:

1. Search chat sessions field.
2. `Projects` heading with Add (`+`).
3. Flexible, full-height, vertically scrollable project list.

Each project row includes a disclosure, project/folder icon, name, hover/focus-only New chat and More actions, and nested chat sessions when expanded. Dragging reorders the entire project with sessions and local state intact.

- Found-path More menu: Reveal/Show in platform file manager, Copy path, Rename, Remove.
- Missing-path More menu: Relocate, Copy path, Rename, Remove.
- A missing path uses lighter italic project-name styling plus accessible state text; no persistent visible `Missing` badge.
- Add and Relocate invoke the native directory chooser. Add appends a project; Relocate repairs the existing project.
- Search filters session titles and hides a project with no matching session.
- Session Remove deletes only that chat and activates a valid fallback if necessary.
- Removing an active project/session or a pending project must never leave detached active state.

## New Chat Lifecycle

New chat creates an unsaved blank thread and does not insert a sidebar session. Center the prompt `What do you want to build in <project name>?`.

Starting this state must cancel Reply, restore ordinary Chat, clear stale composer text and attachments, close artifact focus, and preserve the previously saved thread for later restoration.

Promote the pending chat on the first valid text-only, attachment-only, or combined submission. Derive its title from the first non-empty prompt; if absent, use the first attachment filename. Truncate after 48 characters with an ellipsis. Insert under and activate the originating project before rendering the first exchange.

## Transcript Hierarchy

- User messages align right and omit a visible `You` label.
- Assistant responses align left.
- Every assistant response begins with a full-width work disclosure, followed by an identity row, then content. Use generic Activity/elapsed work for ordinary execution and a Reasoning summary only when the effective route exposes one; never fabricate private reasoning.
- The identity row shows the assistant/C4OS icon and active model for ordinary Chat; Response Artifacts identify the responder as `C4OS` regardless of selected Chat model.
- Expanded work details are unboxed progress paragraphs and muted icon-led activity rows. Do not label them `Work summary` or present private reasoning.
- Assistant bubble and response-artifact surfaces share a family resemblance; the historical proof used a 16px radius with a compact 5px lower-left corner.
- Copy and Reply sit below the bubble on the lower left and appear on hover/focus. They remain keyboard focusable while visually subdued.
- A shared floating down-arrow appears only when the transcript overflows and is more than 24px from its bottom. It moves with the real thread between center and contextual pane.

## Markdown Conversation

Chat and Reply use one toolbar-free source-preserving Markdown editor. Requirements:

- Show canonical source markers while lightly styling headings, emphasis, code, links, block quotes, and list prefixes.
- Enter sends; Shift+Enter inserts or continues a Markdown line.
- Cmd/Ctrl+B and Cmd/Ctrl+I wrap/toggle the relevant source syntax.
- Pasting a URL over selected text creates a Markdown link.
- Raw pasted web addresses are recognized and render as safe links after submission.
- Preserve native undo history.
- Render submitted user and assistant Markdown safely; Copy and Reply retain source Markdown.
- Thinking/working detail stays plain text.

## Attachments And Drop

- Chat paperclip opens a native multi-file picker.
- The entire app accepts file drops only while Chat is active and shows a full-window drop indicator during valid file drag.
- Picker and drop feed one draft-attachment state.
- Draft attachments sit above the editor, are removable, preview images, truncate long names, and show metadata as `<EXT> · <size>`.
- Preserve original one-based reference numbers while displaying newest attachments first.
- Allow attachment-only send.
- Submitted attachments remain a distinct group attached to the user turn, not inside the ordinary text bubble.
- Every draft attachment shows route-specific compatibility such as Ready, Needs Vision, Needs Audio, or Converted. Top-align the file icon, metadata, status, and remove control.
- Adding a file, switching model, or sending runs the same preflight. Unresolved incompatible content remains visible and blocks send; it is never silently dropped.
- A conflict offers explicit Use compatible model, Convert, Remove file, and Cancel actions. Cancel preserves the draft and active model.

## Model And Session Controls

- The model menu filters by All, Vision, Tools, Reasoning, and Audio and shows model capability/context summaries.
- Changing model atomically recomputes dependent controls. Reasoning effort offers Off, Low, Medium, and High only when the effective route supports it; unsupported routes hide the control and clear stale effort.
- The title header retains the centered thread title plus an icon-only Chat information control. Its popover contains runtime, environment, workspace, model, health, and effective context-window usage; adapter details belong to individual run provenance.
- Each assistant response exposes expandable route and effective-capability details while retaining C4OS as the assistant identity.

## Composer Modes

The mode trigger is in the lower toolbar and immediately left of the paperclip in Chat. It opens an upward menu with Chat, Files, Browser, and Terminal, selected state, Escape close, predictable focus return, and `aria-expanded`/menu semantics.

| Mode | Main input | Supporting controls | Primary action |
|---|---|---|---|
| Chat | Markdown editor | Attach, model, approval, branch | Send |
| Files | One path/selection | File-or-folder Browse, branch | Open |
| Browser | Address with boxed globe | None of model/approval/branch | Open |
| Terminal | Command with boxed `$` | None of model/approval/branch | Run |
| Reply | Markdown editor | Quoted-reference strip; no mode trigger | Send |

Mode persists after ordinary submission. Reply remembers the mode, temporarily switches to Chat semantics, and restores the previous mode after cancel or submit.

## Quoted Reply

Reply may target a user message, assistant message, Browser, File, Folder, or Terminal artifact. Show a removable strip with type icon, functional label, and short excerpt/title. Message references use the first few words, not the speaker name.

- Ordinary message reply appends a user/assistant exchange.
- Browser reply updates the target artifact in place and records activity/history.
- File reply creates a proposed edit requiring approval.
- Terminal reply appends a new Terminal artifact in the same shell session; it never executes by mutating the quoted card.

## Panel And Artifact Focus

- Left width defaults to 228px, minimum 180px, maximum 55% while preserving at least 420px center width.
- Pointer drags continuously update the panel and center; cleanup must permit repeated drags.
- Below 992px the panel overlays center and remains resizable.
- Expanding Browser/File/Folder/Terminal replaces the center transcript with that artifact.
- Move the real transcript DOM into a bottom contextual Chat pane inside the left panel; do not clone it.
- Pane defaults to 40vh, retains its session height, and resizes by pointer/keyboard from a top separator up to 60vh.
- Pane header offers non-functional Detach Chat followed by Restore Chat.
- Focused Browser/File/Terminal also offers direct Close so a user can recover while the left panel is collapsed.
- Focusing another artifact swaps it directly; no tabs. Restore returns the transcript to center and the source artifact to its original inline state.
- Preserve unsaved File/Explorer edit state and current Browser/Explorer navigation across focus and restore.

## Conversation Acceptance

Verify project search, add/relocate, menus, rename, remove, sorting, pending chat promotion, session fallback, all composer modes, Markdown shortcuts, paste, attachments, drop, message actions, reply routing, scroll-to-latest, repeated panel resizing, overlay dismissal, artifact focus swaps, pane resizing, collapsed-panel recovery, and restoration of the prior composer mode.
