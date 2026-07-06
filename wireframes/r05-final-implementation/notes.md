# r05 Final Implementation Notes

## Review Round 36 - Batch 5 Runtime Plugin Panels

Date: 2026-07-04

Changed screens, states, copy, layout, or behavior:

- Added focused Terminal plugin routes for user PTY separation, per-chat
  lifecycle, chat deletion cleanup, terminal UI preferences, config.toml
  shell/env/tool-policy split, bounded output, backpressure, and scrollback.
- Added focused Browser plugin routes for navigation/actions, screenshot
  attachment, multi-marker comment capture, prompt evidence bundles,
  clear-after-send, PDF/document preview hosting, invisible runtime state
  hydration, and local-file/security boundaries.
- Added focused Chat Debug routes for off entry, active chat
  timeline, current/historical run selection, typed event detail, redacted
  sensitive fields, retention limits, chat deletion cleanup, and no export.
- Updated the in-artifact coverage matrix for specs 08, 09, 10, and directly
  affected spec 04 runtime/tool-policy overlap.
- After the requirement audit, added an action-state row to
  `#browser-navigation` so Back / Forward / Refresh is visibly represented in
  the Browser panel state, not only available through icon buttons.

What should be reviewed next:

- Panel separation between user PTY Terminal, runtime terminal tool output, and
  Chat Debug visibility.
- Browser evidence capture shape, especially marker/comment bundles and
  clear-after-send behavior.
- Chat Debug redaction clarity, retention limits, and no-export behavior.
- Runtime/tool visibility for invisible Browser actions and C4OS-owned state
  hydration.

Feedback or annotations applied:

- Applied the Batch 5 prompt scope and the existing rule to keep review notes
  out of the rendered shell surface.
- Continued in `wireframes/r05-final-implementation/` and preserved the
  approved r05 header/plugin-panel model.

Simulated or deferred behavior:

- PTY creation, terminal output transport, Browser navigation/capture,
  annotation persistence, Browser hydration, Chat Debug event storage,
  retention pruning, redaction, and cleanup are static review simulations.
- No product code, spec records, or handoff documentation were updated in this
  round.

Open questions:

- None blocking this review round.

Approval path:

- If this review round is approved, the next step is to update specs 08, 09,
  and 10 plus only directly affected 04 documentation with links to the
  approved wireframe routes/states. Do not freeze specs or implement product
  code.

## Review Round 37 - Batch 5 Terminal Scope Correction

Date: 2026-07-04

Changed screens, states, copy, layout, or behavior:

- Applied feedback that the Terminal panel should be the r04 user terminal
  output pane with the agent debug/results pane removed.
- Removed standalone Terminal routes for lifecycle, cleanup, settings, and
  scrollback from the review link set and route map.
- Reduced the Batch 5 link hub to visible panel review routes only:
  Terminal user terminal, Browser navigation/annotations/preview host, Chat
  Debug disabled entry/timeline/event detail, and coverage.
- Removed the explanatory center runtime-state card from Batch 5 routes so the
  center pane reads as normal chat content.
- Updated coverage copy to mark backend/implied requirements as spec-only
  behavior, not standalone wireframe screens.

What should be reviewed next:

- Whether `#terminal-user-pty` now matches the expected r04 terminal minus the
  agent debug/results pane.
- Whether the remaining Browser and Chat Debug routes are genuinely visible
  product surfaces rather than implied/backend state displays.
- Whether the coverage matrix clearly distinguishes visible route evidence from
  spec-only behavior.

Feedback or annotations applied:

- Applied direct chat feedback that trying to display implied/backend behavior
  as separate wireframe screens was confusing and not useful for acceptance.

Simulated or deferred behavior:

- Terminal lifecycle, chat deletion cleanup, terminal preferences, scrollback
  limits, Browser clear-after-send, Browser hydration/security boundaries, and
  Chat Debug retention remain documented as acceptance/spec coverage only.
- No product code, spec records, or handoff documentation were updated.

Open questions:

- Whether Browser security/clear-after-send and Chat Debug retention should
  remain spec-only, or receive visible UI only if a concrete product state is
  later requested.

Approval path:

- If this corrected review round is approved, the next step is to update specs
  08, 09, and 10 plus only directly affected 04 documentation with links to the
  approved visible wireframe routes and coverage rows. Do not freeze specs or
  implement product code.

## Review Round 38 - User Terminal Annotation Removal

Date: 2026-07-05

Changed screens, states, copy, layout, or behavior:

- Removed the `Interactive terminal` status strip from `#terminal-user-pty`.
- The Terminal panel now renders only the terminal output block, matching the
  expected r04 user terminal surface without an agent/debug pane or explanatory
  terminal chrome.

What should be reviewed next:

- Whether `#terminal-user-pty` now reads as an actual user terminal rather than
  an annotated terminal explanation.

Feedback or annotations applied:

- Applied direct feedback that the previous terminal strip was still annotating
  the user terminal.

Simulated or deferred behavior:

- No backend/runtime behavior changed. This remains a static wireframe route.
- No product code, spec records, or handoff documentation were updated.

Open questions:

- None blocking this focused correction.

Approval path:

- If this corrected review round is approved, the next step is still to update
  specs 08, 09, and 10 plus only directly affected 04 documentation with links
  to the approved visible wireframe routes and coverage rows. Do not freeze
  specs or implement product code.

## Review Round 39 - Terminal Spacing and Browser Chrome

Date: 2026-07-05

Changed screens, states, copy, layout, or behavior:

- Reduced the user Terminal panel top padding and added left padding so
  `#terminal-user-pty` reads as a plain user terminal with a cleaner text inset.
- Repaired a missing CSS brace that kept later Browser and Terminal styles from
  applying in rendered Browser QA.
- Added document-relative cache tokens to the CSS and JS asset references so the
  review browser loads the current artifact.
- Updated Browser chrome to the requested control set: Back, Forward, Refresh,
  centered `iamawesome.com`, Screenshot, Annotate, and Browser menu.
- Added visible Browser toolbar-menu and page right-click-menu states matching
  the provided menu contents.
- Renamed Browser frame state classes to `browser-mode-*` to prevent collision
  with the actual page context-menu class.
- Tightened Browser toolbar spacing and anchored the Browser menu inside the
  approved r05 right plugin panel.

What should be reviewed next:

- Whether `#terminal-user-pty` now has the expected terminal spacing and no
  agent/debug pane.
- Whether `#browser-navigation`, `#browser-menu`, and
  `#browser-page-context-menu` match the requested Browser controls and menu
  shapes within the existing r05 plugin-panel model.

Feedback or annotations applied:

- Applied direct feedback from the three Browser reference screenshots and the
  requested Terminal padding adjustment.

Simulated or deferred behavior:

- Browser navigation, screenshot, annotation, clear browsing data, zoom, find,
  device toolbar, Browser settings, and Inspect remain static visual states.
- No product code, spec records, or handoff documentation were updated.

Open questions:

- None blocking this focused visual correction.

Approval path:

- If this corrected review round is approved, the next step is still to update
  specs 08, 09, and 10 plus only directly affected 04 documentation with links
  to the approved visible wireframe routes and coverage rows. Do not freeze
  specs or implement product code.

## Review Round 40 - Browser Wireframe Chrome

Date: 2026-07-06

Changed screens, states, copy, layout, or behavior:

- Changed Browser navigation chrome from black to the r05 gray/white wireframe
  treatment.
- Changed Browser toolbar menu and page right-click menu panels to light
  gray/white surfaces to keep Browser states consistent with the grayscale
  artifact.
- Added a dedicated comment-plus icon and used it for the Annotate toolbar
  control.
- Bumped the document-relative CSS/JS asset token to `batch5-r41`.

What should be reviewed next:

- Whether `#browser-navigation` now matches the gray/white wireframe format.
- Whether the Annotate button reads as a comment bubble with a plus in the
  center and aligns with the other Browser toolbar controls.
- Whether `#browser-menu` and `#browser-page-context-menu` still read clearly
  after the light-menu update.

Feedback or annotations applied:

- Applied direct feedback that the Browser navigation chrome was too black for
  the wireframe format and that the annotation icon needed to be a centered
  comment-plus control.

Simulated or deferred behavior:

- Browser navigation, screenshot, annotation, menu actions, and Inspect remain
  static visual states.
- No product code, spec records, or handoff documentation were updated.

Open questions:

- None blocking this focused visual correction.

Approval path:

- If this corrected review round is approved, the next step is still to update
  specs 08, 09, and 10 plus only directly affected 04 documentation with links
  to the approved visible wireframe routes and coverage rows. Do not freeze
  specs or implement product code.

## Review Round 41 - Browser Menu Access

Date: 2026-07-06

Changed screens, states, copy, layout, or behavior:

- Wired the three-dot Browser toolbar button in `#browser-navigation` to open
  the Browser menu state.
- Wired right-click on the Browser document page in `#browser-navigation` to
  open the page context menu state.
- Added a scoped Browser document marker for the right-click interaction.
- Bumped the document-relative CSS/JS asset token to `batch5-r42`.

What should be reviewed next:

- Whether clicking the Browser menu button from `#browser-navigation` opens the
  Browser context menu.
- Whether right-clicking the Browser document page from `#browser-navigation`
  opens the page context menu.
- Whether both menus remain visually aligned with the gray/white r05 wireframe
  format.

Feedback or annotations applied:

- Applied direct feedback that the Browser navigation context menu and
  right-click context menu were missing from the Browser navigation review
  state.

Simulated or deferred behavior:

- Menu commands remain static visual states; this only wires the review
  interactions between Browser states.
- No product code, spec records, or handoff documentation were updated.

Open questions:

- None blocking this focused interaction correction.

Approval path:

- If this corrected review round is approved, the next step is still to update
  specs 08, 09, and 10 plus only directly affected 04 documentation with links
  to the approved visible wireframe routes and coverage rows. Do not freeze
  specs or implement product code.

## Review Round 42 - Browser Menu Typography

Date: 2026-07-06

Changed screens, states, copy, layout, or behavior:

- Reduced Browser toolbar context menu and page right-click context menu type
  to the r05 wireframe UI size.
- Set both Browser menus to `var(--font-ui)` so the right-click menu no longer
  inherits the Browser document serif type.
- Tightened menu row height, divider spacing, and zoom-control type/height.
- Bumped the document-relative CSS/JS asset token to `batch5-r43`.

What should be reviewed next:

- Whether the Browser toolbar context menu and page right-click context menu
  now match the surrounding r05 wireframe typography.
- Whether the smaller context menus still read clearly inside the Browser
  plugin panel.

Feedback or annotations applied:

- Applied direct feedback that the Browser navigation context menu and
  right-click context menu font size was too large.

Simulated or deferred behavior:

- Menu commands remain static visual states; this round changes typography and
  spacing only.
- No product code, spec records, or handoff documentation were updated.

Open questions:

- None blocking this focused typography correction.

Approval path:

- If this corrected review round is approved, the next step is still to update
  specs 08, 09, and 10 plus only directly affected 04 documentation with links
  to the approved visible wireframe routes and coverage rows. Do not freeze
  specs or implement product code.

## Review Round 43 - Browser Menu Weight

Date: 2026-07-06

Changed screens, states, copy, layout, or behavior:

- Removed bold styling from Browser toolbar context menu and page right-click
  context menu item text.
- Preserved the compact `13px` `var(--font-ui)` menu typography from Review
  Round 42.
- Bumped the document-relative CSS/JS asset token to `batch5-r44`.

What should be reviewed next:

- Whether the Browser toolbar context menu and page right-click context menu
  now match the regular-weight r05 wireframe typography.
- Whether the menus still have enough hierarchy without bold labels.

Feedback or annotations applied:

- Applied direct feedback to remove bold text from the Browser navigation
  context menu and right-click context menu.

Simulated or deferred behavior:

- Menu commands remain static visual states; this round changes item text
  weight only.
- No product code, spec records, or handoff documentation were updated.

Open questions:

- None blocking this focused weight correction.

Approval path:

- If this corrected review round is approved, the next step is still to update
  specs 08, 09, and 10 plus only directly affected 04 documentation with links
  to the approved visible wireframe routes and coverage rows. Do not freeze
  specs or implement product code.

## Review Round 44 - Chat Debug r04 Correction

Date: 2026-07-06

Changed screens, states, copy, layout, or behavior:

- Changed the Debug header icon to open the active `#debug` panel instead of
  the disabled-entry state.
- Reworked the active Chat Debug panel as an r04-style command log plus result
  area.
- Added visible CLI command, tool call, tool result, and approval rows.
- Tightened `#debug-disabled-entry` so the pill and settings button do not
  stretch into oversized shapes.
- Added `#debug` to the Batch 5 route hub.
- Bumped the document-relative CSS/JS asset token to `batch5-r45`.

What should be reviewed next:

- Whether Debug now looks like the expected agent debug surface for CLI
  commands, tool calls, and results.
- Whether the Debug panel is clearly separate from the user Terminal panel.
- Whether the disabled-entry state is acceptable as a secondary route only.

Feedback or annotations applied:

- Applied direct screenshot feedback that Agent/Chat Debug was not supposed to
  open on the disabled card and should be based on the r04 debug/results
  pattern.

Simulated or deferred behavior:

- Debug command log, tool calls, and results remain static wireframe content.
- No product code, spec records, or handoff documentation were updated.

Open questions:

- None blocking this focused Chat Debug correction.

Approval path:

- If this corrected review round is approved, the next step is still to update
  specs 08, 09, and 10 plus only directly affected 04 documentation with links
  to the approved visible wireframe routes and coverage rows. Do not freeze
  specs or implement product code.

## Review Round 45 - Chat Debug Label Removal

Date: 2026-07-06

Changed screens, states, copy, layout, or behavior:

- Removed visible annotation-like type labels from the active Chat Debug
  results panel.
- Replaced the rows with realistic `terminal.run` and `browser.screenshot`
  records.
- Added visible command/tool parameters and returned result payloads.
- Updated the active command log to show the same command, stdout, exit code,
  tool-call parameters, and tool-call result content.
- Bumped the document-relative CSS/JS asset token to `batch5-r46`.

What should be reviewed next:

- Whether `#debug` now reads as an intended product debug surface, not an
  annotated wireframe.
- Whether the sample CLI command/result and tool call/result are concrete
  enough for acceptance.

Feedback or annotations applied:

- Applied direct feedback that the previous Agent Debug wireframe violated the
  no-annotation rule by describing record types instead of showing realistic
  debug records.

Simulated or deferred behavior:

- Debug records remain static wireframe content.
- No product code, spec records, or handoff documentation were updated.

Open questions:

- None blocking this focused Chat Debug correction.

Approval path:

- If this corrected review round is approved, the next step is still to update
  specs 08, 09, and 10 plus only directly affected 04 documentation with links
  to the approved visible wireframe routes and coverage rows. Do not freeze
  specs or implement product code.

## Review Round 46 - Chat Debug Secondary States

Date: 2026-07-06

Changed screens, states, copy, layout, or behavior:

- Kept `#debug-disabled-entry` as a compact off state with a settings path.
- Removed the extra disabled notice from the center composer dock.
- Reworked `#debug-timeline` as a distinct current/historical run selector.
- Added selected-run event rows to `#debug-timeline`.
- Bumped the document-relative CSS/JS asset token to `batch5-r48`.

What should be reviewed next:

- Whether `#debug-disabled-entry` should stay as the Chat Debug off state.
- Whether `#debug-timeline` is now useful enough as a separate run-history
  review state.
- Whether both states avoid backend/background explanatory content.

Feedback or annotations applied:

- Applied direct feedback to fix these two states without wireframing backend
  or background functionality.

Simulated or deferred behavior:

- Run selection is static wireframe content.
- No product code, spec records, or handoff documentation were updated.

Open questions:

- None blocking this focused correction.

Approval path:

- If this corrected review round is approved, the next step is still to update
  specs 08, 09, and 10 plus only directly affected 04 documentation with links
  to the approved visible wireframe routes and coverage rows. Do not freeze
  specs or implement product code.

## Review Round 47 - Remove Chat Debug Off Route

Date: 2026-07-06

Changed screens, states, copy, layout, or behavior:

- Removed `#debug-disabled-entry` from the route table.
- Removed the disabled Chat Debug panel component and unused disabled notice.
- Removed `#debug-disabled-entry` from the Batch 5 link hub and README route
  table.
- Moved disabled-by-default Chat Debug coverage into the spec/settings behavior
  coverage row instead of treating it as a standalone wireframe screen.
- Bumped the document-relative CSS/JS asset token to `batch5-r49`.

What should be reviewed next:

- Whether the remaining Chat Debug review routes should be limited to
  `#debug`, `#debug-timeline`, and `#debug-event-detail`.
- Whether disabled-by-default plugin visibility is now correctly treated as
  Settings/spec behavior instead of panel UI.

Feedback or annotations applied:

- Applied direct feedback that the Chat Debug off route represents background
  plugin state and cannot be represented as a product wireframe panel.

Simulated or deferred behavior:

- Disabled plugin behavior remains a spec/settings concern only.
- No product code, spec records, or handoff documentation were updated.

Open questions:

- None blocking this focused route removal.

Approval path:

- If this corrected review round is approved, the next step is still to update
  specs 08, 09, and 10 plus only directly affected 04 documentation with links
  to the approved visible wireframe routes and coverage rows. Do not freeze
  specs or implement product code.

## Review Round 48 - Move Browser Annotation Flow To Markdown

Date: 2026-07-06

Changed screens, states, copy, layout, or behavior:

- Removed `#browser-annotations` from the route table.
- Removed the Browser annotation route from the Batch 5 link hub and README
  route table.
- Removed rendered annotation-marker/comment UI and related CSS.
- Removed the annotation control from `#browser-preview-host`.
- Moved Browser annotation behavior into `browser-annotations.md`.
- Replaced visible annotation sample data in prompt attachments and Debug event
  detail with file/screenshot/evidence samples.
- Bumped the document-relative CSS/JS asset token to `batch5-r50`.

What should be reviewed next:

- Whether the remaining Browser review routes should be limited to navigation,
  toolbar menu, page context menu, and preview host.
- Whether `#browser-preview-host` now reads as document preview hosting only.
- Whether `browser-annotations.md` is the right place for non-rendered
  annotation behavior.

Feedback or annotations applied:

- Applied direct feedback to remove `#browser-annotations`, remove annotations
  from `#browser-preview-host`, and move annotation behavior into Markdown.

Simulated or deferred behavior:

- Browser annotation capture remains markdown/spec behavior only in this review
  batch.
- No product code, spec records, or handoff documentation were updated.

Open questions:

- None blocking this focused Browser route cleanup.

Approval path:

- If this corrected review round is approved, the next step is still to update
  specs 08, 09, and 10 plus only directly affected 04 documentation with links
  to the approved visible wireframe routes and coverage rows. Do not freeze
  specs or implement product code.

## Review Round 49 - Remove Preview Host Helper Labels

Date: 2026-07-06

Changed screens, states, copy, layout, or behavior:

- Removed the `Browser-native PDF preview` caption from
  `#browser-preview-host`.
- Removed the `DOCX/XLSX rendered by document-family plugins` and
  `Browser hosts rendered output` pill labels from `#browser-preview-host`.
- Removed unused document-boundary strip CSS.
- Added the removed helper text to `browser-annotations.md` as Markdown-only
  review context.
- Bumped the document-relative CSS/JS asset token to `batch5-r51`.

What should be reviewed next:

- Whether `#browser-preview-host` now reads as a plain document preview
  surface without helper/callout text.
- Whether the document preview boundary notes are acceptable in Markdown only.

Feedback or annotations applied:

- Applied direct screenshot feedback that the highlighted helper/callout labels
  were still annotation-like text in `#browser-preview-host`.

Simulated or deferred behavior:

- Document preview boundary behavior remains Markdown/spec context only in this
  review batch.
- No product code, spec records, or handoff documentation were updated.

Open questions:

- None blocking this focused Browser preview cleanup.

Approval path:

- If this corrected review round is approved, the next step is still to update
  specs 08, 09, and 10 plus only directly affected 04 documentation with links
  to the approved visible wireframe routes and coverage rows. Do not freeze
  specs or implement product code.

## Review Round 50 - Debug Event Detail Cleanup

Date: 2026-07-06

Changed screens, states, copy, layout, or behavior:

- Removed the `Typed event` pill from `#debug-event-detail`.
- Replaced the `Back to timeline` text link with an icon-only X control that
  links to `#debug-timeline`.
- Removed the bottom no-export callout from `#debug-event-detail`.
- Updated the route subtitle and coverage row wording to avoid visible
  `Typed event` and no-export callout copy.
- Bumped the document-relative CSS/JS asset token to `batch5-r52`.

What should be reviewed next:

- Whether `#debug-event-detail` now reads as a plain event detail panel.
- Whether the X control is enough to return to the timeline without visible
  instructional text.
- Whether removing the bottom callout resolves the annotation concern.

Feedback or annotations applied:

- Applied screenshot feedback to remove the event-type pill, replace the
  timeline back text with an X icon, and remove the bottom annotation-like
  no-export message.

Simulated or deferred behavior:

- No-export behavior remains represented by the absence of export controls in
  the Chat Debug panel.
- No product code, spec records, or handoff documentation were updated.

Open questions:

- None blocking this focused Debug event-detail cleanup.

Approval path:

- If this corrected review round is approved, the next step is still to update
  specs 08, 09, and 10 plus only directly affected 04 documentation with links
  to the approved visible wireframe routes and coverage rows. Do not freeze
  specs or implement product code.
