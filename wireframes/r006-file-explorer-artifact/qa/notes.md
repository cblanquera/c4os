# Visual QA — r006 Round 1

- Viewport reviewed: 1280 × 720 desktop.
- Files composer:
  - Browse exposed separate Choose file and Choose folder actions.
  - Choose folder set the illustrative path to `~/project`, updated the primary action to Open folder, and preserved Files mode after submission.
  - Folder submission appended one File Explorer artifact with six read-only folder/file rows and one Expand action.
- File Explorer:
  - Expanded into the complete center workspace while the thread moved into the contextual left Chat pane.
  - Folder rows remained keyboard-addressable buttons; opening `src` updated the breadcrumbs from `project` to `project / src`.
  - Detach appeared immediately left of Restore Chat and deliberately produced no state change when activated.
- Focused File:
  - Header showed `project / docs / notes.md` breadcrumbs with no repeated filename/version block.
  - Sample content contained 34 lines and exceeded the available editor height, producing a real vertical scroll range.
  - Edit mode revealed 34 fixed line numbers. Scrolling the editor to 220px synchronized the line-number gutter to 220px.
  - Computed focused-editor padding was approximately half the previous treatment at this viewport.
  - Clicking the `docs` breadcrumb replaced the focused File view with a File Explorer view for `project / docs`.
- Regression checks: composer mode chooser stayed hidden during artifact focus, Terminal retained zero Expand controls, no horizontal document overflow appeared, and all scripted interactions completed without runtime errors.

## Captures

- `round-1-file-explorer.png`
- `round-1-file-editor.png`

# Visual QA — r006 Round 2

- Viewport reviewed: 1280 × 720 desktop.
- Inline File Explorer:
  - Activating `docs` updated the existing artifact in place without entering artifact focus.
  - Breadcrumbs changed from `project` to `project / docs`.
  - The six root entries were replaced by four distinct children: `guides`, `references`, `notes.md`, and `architecture.md`.
  - Returning through the `project` breadcrumb and activating `index.html` replaced the rows with a constrained HTML read preview, `project / index.html` breadcrumbs, and a Read only status.
- Expanded File Explorer:
  - Activating `docs` updated the full center listing to the same four path-specific children and updated the footer count to four items.
  - Activating `notes.md` opened a full-height, 34-line read-only file surface at `project / docs / notes.md`.
  - Parent breadcrumb activation returned from file read mode to the appropriate folder listing.
  - Expanding an inline `index.html` preview opened the same file directly in center read mode.
- Regression checks: the contextual left Chat pane remained available, the composer stayed fixed and Chat-only during focus, Terminal retained zero Expand controls, and no horizontal overflow appeared.

## Captures

- `round-2-folder-children.png`
- `round-2-file-read.png`

# Visual QA — r006 Round 3

- Viewport reviewed: active desktop browser viewport.
- File-open semantics:
  - Activating `index.html` from the inline File Explorer opened the file in a non-editing `Viewing` state.
  - The open file retained a visible Edit action; Edit exposed line numbers plus Cancel and Save.
  - Saving inline content returned to `Viewing` and preserved the updated content when the same artifact expanded.
  - The expanded file reused the same document surface, opened non-editing, retained Edit, and returned to `Viewing` after Save.
- Reuse and cleanup audit:
  - Inline and expanded folder/file explorer views are rendered by the shared `explorerSurfaceMarkup` path with shared folder and file body helpers.
  - Explorer file edit input, line numbering, Cancel, Save, and scroll synchronization use the same state handlers in both contexts.
  - Retired right-panel tab and expanded-Terminal selectors were removed.
- Regression checks:
  - Focused artifact mode kept the composer mode chooser at computed `display: none`.
  - Terminal retained zero Expand controls.
  - No horizontal document overflow or browser console errors appeared.

## Capture

- `round-3-open-file.png`

# Visual QA — r006 Round 4

- Contenteditable File editor:
  - The inline File editor renders as a `DIV` with `contenteditable="true"`, not a textarea.
  - The 34-line sample produced matching 701px editor and line-number heights without the previous internal textarea height mismatch.
  - Browser interaction could replace content and Save preserved it in the viewing preview.
- Breadcrumbs and explorer:
  - Inline File displayed `project / docs / notes.md` breadcrumbs.
  - Activating `docs` opened a focused four-row File Explorer at `project / docs`.
  - The expanded folder view had no status/item-count footer.
- Responsive left panel:
  - At 826 × 819, the resize handle remained visible.
  - Dragging the handle changed the overlay panel from 228px to 313px while the center workspace remained full-width behind the overlay.
- Browser composer and regressions:
  - Browser address mode exposed one browser-window rectangle and no globe circle in its leading icon.
  - Terminal retained zero Expand controls and no horizontal overflow or console errors appeared.

## Capture

- `round-4-contenteditable.png`

# Visual QA — r006 Round 5

- Bounded response artifacts:
  - All inline artifact surfaces computed the shared `440px` maximum at the reviewed desktop viewport with `overflow-y: auto`.
  - The edited File response measured 440px high against 788px of scroll content, confirming a real internal scroll range.
  - Its contenteditable document remained 701px tall while the artifact header and footer computed as sticky.
  - Browser and Terminal artifacts remained naturally shorter than the shared maximum.
- Inline File breadcrumbs:
  - Activating `docs` from `project / docs / notes.md` kept the response inside the thread and did not enter artifact focus.
  - The same response changed to a Folder artifact at `project / docs` with four folder/file rows and the shared Folder navigation behavior.
- Responsive left panel:
  - At 800 × 819, the left panel used overlay positioning while the center workspace remained 800px wide and started at x=0.
  - Clicking the center workspace changed the open left-panel state to closed.
- Browser icon and regressions:
  - Browser mode and the Browser address input had identical globe SVG markup.
  - Terminal retained zero Expand controls, no horizontal document overflow appeared, and the browser console contained no errors.

## Capture

- `round-5-artifact-scroll.png`

# Visual QA — r006 Round 6

- File edit actions:
  - Inline and expanded File edit modes exposed a `Discard changes to notes.md` control using the shared trash SVG, followed by Save.
  - The expanded artifact retained a separate Close control at the far right.
  - Discard remained reversible and returned the file to its prior viewing state.
- File surface cleanup:
  - Inline and expanded File views contained zero File status footers in both viewing and editing states.
  - The focused editor preserved 34 newline-separated content lines and displayed 34 matching line numbers with computed `white-space: pre-wrap`.
- Contextual Chat pane:
  - User and assistant message bodies both computed to 13px text with a 19.5px line height while hosted in the left pane.
- Regression checks:
  - The bounded inline File editor remained internally scrollable, Terminal retained zero Expand controls, no horizontal overflow appeared, and the browser console contained no errors.

## Capture

- `round-6-file-controls.png`

# Visual QA — r006 Round 7

- Focused File Explorer file:
  - The expanded center file retained its editing controls.
  - Its matching File response in the left Chat pane exposed zero Edit, Discard, or Save controls.
  - The left-pane response remained visible as read-only conversation context.
- Regression checks: Terminal retained zero Expand controls, no horizontal overflow appeared, and the browser console contained no errors.

# Visual QA — r006 Round 8

- User prompt actions:
  - All four seeded user prompts rendered exactly one shared message-action group.
  - Each group contained one Copy and one Reply control.
  - Browser, File, and Terminal command prompts exposed their controls on hover/focus just like the initial chat prompt.
- Regression checks: Newly appended user messages still use the same shared action helper, no duplicate action groups appeared, and the browser console contained no errors.

# Visual QA — r006 Round 9

- Main conversation:
  - The circular down-arrow appeared when the thread was scrolled away from the bottom.
  - Activating it smoothly returned the thread to the bottom and hid the control within the 24px threshold.
- Contextual left Chat pane:
  - Expanding an artifact moved the same control beside the same conversation into the left-pane host.
  - The control used the pane-specific bottom offset and followed the left thread’s independent scroll position.
- Regression checks: The main and left-pane hosts never rendered duplicate scroll controls, user prompt actions remained intact, and the browser console contained no errors.

# Visual QA — r006 Round 10

- Focused File viewing state: The document surface computed `overflow-x/y: auto`, retained a constrained client height, and exposed a real internal vertical scroll range.
- Focused File editing state: The same surface retained automatic overflow; scrolling it updated the fixed line-number gutter to the same scroll position.
- Regression checks: The File header and composer remained fixed outside the document scroller, and the browser console contained no errors.
