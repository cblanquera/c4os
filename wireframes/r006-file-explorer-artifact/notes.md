# r006 Review Notes

## Round 1 — File Explorer Artifact

- Date: 2026-07-15
- Phase: Conceptual wireframes.
- Revision: `r006-file-explorer-artifact`.
- Feedback applied:
  - Add a future Detach Chat affordance immediately left of Restore Chat without implementing native-window behavior.
  - Reduce focused File content padding by approximately half.
  - Add fixed left-side line numbers while editing a focused File and expand the sample content enough to test scrolling.
  - Replace the focused File filename/version block with clickable breadcrumbs; clicking a folder opens File Explorer.
  - Let Files mode choose either a file or folder and create a File Explorer response artifact for folder selections.
- Changed behavior:
  - Files Browse opens a compact choice menu for File or Folder.
  - Folder submission appends a read-only File Explorer artifact with breadcrumb navigation and folder/file rows.
  - Expanding File Explorer uses the center workspace and the same contextual left Chat pane behavior as Browser and File.
  - Focused File edit mode adds a synchronized fixed line-number gutter while the content scrolls independently.
  - Focused File breadcrumbs can transition directly into a read-only File Explorer view.
- Simulated or deferred behavior: Filesystem contents and selection are illustrative; choosing File or Folder fills a sample path. Detach Chat is visual only and opening a native desktop window is explicitly out of scope.
- Review focus: Browse-choice clarity, File Explorer information density, breadcrumb behavior, long-file scroll treatment, and placement of Detach beside Restore.
- Open questions: None blocking this round.
- Approval path: Approval keeps `r006` as the active wireframe revision; further visual or behavior refinements remain review rounds inside this folder unless they materially change the artifact model again.

## Round 2 — Explorer Navigation And File Read Mode

- Date: 2026-07-15
- Phase: Conceptual wireframes.
- Revision: `r006-file-explorer-artifact`.
- Feedback applied:
  - Make folder rows show a realistic sample of the selected folder’s children.
  - Make file rows open the selected file in read mode.
  - Apply both behaviors to inline File Explorer artifacts and the expanded center workspace.
- Changed behavior:
  - Root, `docs`, `src`, and `wireframes` paths now render distinct illustrative child collections and updated item counts.
  - Inline folder navigation updates the original response artifact in place instead of automatically expanding it.
  - Inline file selection replaces the folder rows with a constrained read-only file preview and breadcrumbs.
  - Expanded file selection replaces the center explorer with a full-height read-only file surface; activating a parent breadcrumb returns to that folder.
- Simulated or deferred behavior: Folder contents and file source text remain illustrative in-memory data; no local filesystem is read.
- Review focus: Whether in-place folder navigation and read-only file opening feel natural in both inline and expanded artifact sizes.
- Open questions: None blocking this round.
- Approval path: Approval keeps `r006` as the active wireframe revision; further refinements remain review rounds inside this folder unless the artifact model changes materially.

## Round 3 — Open File State And Artifact Reuse Audit

- Date: 2026-07-15
- Phase: Conceptual wireframes.
- Revision: `r006-file-explorer-artifact`.
- Feedback applied:
  - Interpret opening a file as a normal viewing state rather than a permanently read-only artifact.
  - Keep Edit available after a file is opened from File Explorer.
  - Audit CSS and JavaScript so inline response artifacts and expanded artifact views reuse the same rendering and state functions where their behavior matches.
- Changed behavior:
  - Files opened from inline or expanded File Explorer now show a pencil action while remaining non-editing by default.
  - Edit reveals line numbers plus Cancel and Save; saving returns to the viewing state and preserves the updated in-memory content.
  - Inline and expanded folder/file explorer surfaces now share `explorerSurfaceMarkup`, `explorerFolderBodyMarkup`, and `explorerFileBodyMarkup` instead of maintaining separate expanded-only file markup.
  - Shared explorer-file CSS now drives both response-bubble and full-workspace sizes, with context-specific sizing modifiers only where necessary.
  - Removed unreachable right-panel tab and expanded-Terminal CSS left behind by earlier revisions.
- Simulated or deferred behavior: File contents, edits, versions, and folder listings remain illustrative in-memory state; no local filesystem is read or written.
- Review focus: Whether the default open state clearly differs from Edit, whether Edit/Cancel/Save work consistently inline and expanded, and whether the shared surface treatment remains visually appropriate at both sizes.
- Open questions: None blocking this round.
- Approval path: Approval keeps `r006` as the active wireframe revision; further refinements remain review rounds inside this folder unless the artifact model changes materially.

## Round 4 — Contenteditable Files And Responsive Panel Resize

- Date: 2026-07-15
- Phase: Conceptual wireframes.
- Revision: `r006-file-explorer-artifact`.
- Feedback applied:
  - Replace the inline File textarea with a contenteditable surface so its height follows the document content.
  - Show the same navigable file breadcrumbs on inline File artifacts that appear in the expanded workspace.
  - Remove the expanded File Explorer `Browsing / item count` footer.
  - Restore left-panel resizing below the narrow desktop breakpoint.
  - Replace the Browser composer address icon with a browser-window icon.
- Changed behavior:
  - Inline, expanded, and explorer-opened File editors use contenteditable document surfaces with shared text extraction and line-number synchronization.
  - Inline File breadcrumbs show `project / docs / notes.md`; selecting a folder opens that directory in the focused File Explorer workspace.
  - Expanded folder listings end after their final row, while inline folder artifacts retain their compact status footer.
  - The left resize handle remains visible and draggable in the narrow overlay layout instead of being disabled by the responsive breakpoint.
  - Browser address mode now uses a browser-window glyph inside the boxed leading control.
- Simulated or deferred behavior: File content and writes remain in-memory; breadcrumb folders and files continue to use illustrative data. Narrow layouts continue to treat the left panel as an overlay while allowing its width to change.
- Review focus: Contenteditable auto-height, breadcrumb placement/navigation, the footer-free expanded folder list, narrow-width resizing, and the Browser address icon.
- Open questions: None blocking this round.
- Approval path: Approval keeps `r006` as the active wireframe revision; further refinements remain review rounds inside this folder unless the artifact model changes materially.

## Round 5 — Bounded Artifacts And Inline Breadcrumb Navigation

- Date: 2026-07-15
- Phase: Conceptual wireframes.
- Revision: `r006-file-explorer-artifact`.
- Feedback applied:
  - Give every inline response artifact a maximum height with internal scrolling for longer content.
  - Close an open overlay left panel when the center workspace is clicked below 992px.
  - Use the exact Browser mode globe icon beside the Browser address input.
  - Keep inline File breadcrumb navigation inside the transcript instead of expanding into artifact focus.
- Changed behavior:
  - Browser, File, Folder, and Terminal response surfaces share a responsive maximum height; longer surfaces scroll internally while their headers and footers remain visible.
  - The responsive overlay breakpoint is now 992px, and a center-workspace click dismisses an open left panel without blocking the clicked workspace action.
  - Browser mode and its address field now use the same globe SVG geometry.
  - Selecting a parent folder from an inline File breadcrumb converts that response card in place to the matching Folder artifact and reuses the same folder rows, breadcrumbs, and file-opening behavior as an existing Folder response.
- Simulated or deferred behavior: Artifact data, folder contents, and edits remain illustrative in-memory state. File writes still follow the conceptual approval path and do not touch the local filesystem.
- Review focus: Internal artifact scrolling, sticky artifact chrome, overlay dismissal below 992px, Browser icon consistency, and in-transcript File-to-Folder breadcrumb navigation.
- Open questions: None blocking this round.
- Approval path: Approval keeps `r006` as the active wireframe revision; further refinements remain review rounds inside this folder unless the artifact model changes materially.

## Round 6 — File Edit Action Clarity And Compact Left Chat

- Date: 2026-07-15
- Phase: Conceptual wireframes.
- Revision: `r006-file-explorer-artifact`.
- Feedback applied:
  - Replace the first edit-mode X with a trash icon in expanded and inline File artifacts.
  - Remove the File `Viewing / MD` footer in viewing and editing states.
  - Reduce user and assistant message text size inside the contextual left Chat pane.
- Changed behavior:
  - Inline File, expanded File, and File Explorer-opened File editors use the same trash icon for the reversible Discard changes action, followed by Save; the separate far-right X still closes the expanded artifact.
  - File response and expanded surfaces no longer render a status/file-type footer in either viewing or editing state.
  - User and assistant message bodies render at 13px with a 1.5 line height only while the thread is inside the left Chat pane.
  - The focused contenteditable File surface explicitly preserves newlines with `pre-wrap`, keeping its 34 text lines aligned with the fixed line-number gutter.
- Simulated or deferred behavior: Trash discards the current unsaved edit; it does not delete the file. File changes remain in memory and do not touch the local filesystem.
- Review focus: Distinction between Discard, Save, and Close; the footer-free File canvas; and readability of the denser left-pane conversation.
- Open questions: None blocking this round.
- Approval path: Approval keeps `r006` as the active wireframe revision; further refinements remain review rounds inside this folder unless the artifact model changes materially.

## Round 7 — Read-Only Left-Pane File Cards

- Date: 2026-07-15
- Phase: Conceptual wireframes.
- Revision: `r006-file-explorer-artifact`.
- Feedback applied: Remove the File edit action from response artifacts while the conversation is hosted in the contextual left Chat pane.
- Changed behavior: File cards opened through File Explorer now follow the same focused-workspace rule as direct File cards: Edit, Discard, and Save controls are hidden in the left pane while the expanded center artifact owns editing.
- Simulated or deferred behavior: The left-pane response remains a navigable transcript reference; file editing stays available in the expanded center workspace and remains in-memory only.
- Review focus: Confirm that the left Chat pane reads as conversation context rather than a competing editing surface.
- Open questions: None.
- Approval path: This completes the requested `r006` review refinements and is ready for a local commit.

## Round 8 — Complete User Prompt Actions

- Date: 2026-07-15
- Phase: Conceptual wireframes.
- Revision: `r006-file-explorer-artifact`.
- Feedback applied: Restore missing Copy and Reply hover controls on some user prompts.
- Changed behavior: A shared initialization pass now adds the existing message-action component to any seeded user message that does not already have it. Seeded Browser, File, and Terminal command prompts now match the first prompt and all newly submitted prompts.
- Simulated or deferred behavior: Copy uses the browser clipboard when available; Reply continues to populate the conceptual quoted-reply composer state.
- Review focus: Hover or keyboard-focus each user prompt and confirm the same two actions appear without duplication.
- Open questions: None.
- Approval path: Approval keeps this refinement in `r006`; it remains uncommitted until the user requests another commit.

## Round 9 — Shared Scroll-To-Latest Control

- Date: 2026-07-15
- Phase: Conceptual wireframes.
- Revision: `r006-file-explorer-artifact`.
- Feedback applied: Add a floating circular down-arrow control to the main conversation and contextual left Chat pane, hidden whenever the conversation is already at the bottom.
- Changed behavior:
  - One shared scroll control moves with the conversation between the main thread host and left-pane host.
  - The control observes scrolling, thread resizing, viewport resizing, and appended messages.
  - It appears only when the active conversation has overflow and is more than 24px from the bottom; activation smoothly scrolls to the latest message and then hides the control.
  - Main-thread placement floats above the fixed composer, while left-pane placement floats above the pane’s bottom edge.
- Simulated or deferred behavior: Scrolling uses native smooth scrolling; there is no unread-count badge in this wireframe round.
- Review focus: Visibility threshold, circular affordance placement in both thread modes, and whether the smooth return to the latest message feels predictable.
- Open questions: None.
- Approval path: Approval keeps this refinement in `r006`; it remains uncommitted until the user requests another commit.

## Round 10 — Focused File Internal Scrolling

- Date: 2026-07-15
- Phase: Conceptual wireframes.
- Revision: `r006-file-explorer-artifact`.
- Feedback applied: Set the expanded File document surface to `overflow: auto` in both viewing and editing states.
- Changed behavior: The shared `.pane-file-editor` now owns vertical and horizontal overflow with contained overscroll, keeping the File header and fixed composer outside the document scroll area. Edit-mode scrolling continues to synchronize the fixed line-number gutter.
- Simulated or deferred behavior: File contents remain illustrative and in-memory.
- Review focus: Internal scrolling in View and Edit, preserved header/composer placement, and line-number alignment while scrolling.
- Open questions: None.
- Approval path: Approval keeps this refinement in `r006`; it remains uncommitted until the user requests another commit.
