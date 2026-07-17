# Workspace Polish Review Notes

## Round 1 — Composer Dock Divider

- Date: 2026-07-15
- Phase: Conceptual wireframe polish.
- Revision: `r008-workspace-polish`.
- Feedback applied: Create a new revision for final polish work and add a top border to the `.composer-dock` region.
- Changed behavior: None. The approved r007 layout and interactions are copied forward unchanged.
- Visual change: Added a one-pixel `--gray-300` top border across the composer dock to distinguish the fixed composer region from the workspace above it.
- Simulated or deferred behavior: All simulation and production deferrals from r007 remain unchanged.
- Review focus: Confirm that the divider provides enough separation without making the composer dock feel like a heavy panel.
- Open questions: None blocking this round.
- Approval path: Approval keeps this divider in r008 and allows the next requested polish item to be applied as another review round in the same revision.

## Round 2 — Dynamic Composer Boundary

- Date: 2026-07-15
- Phase: Conceptual wireframe polish.
- Revision: `r008-workspace-polish`.
- Feedback applied: The workspace stage must stop above the composer dock so its native scrollbar is complete, including when multiline prompt input expands the dock upward.
- Changed behavior:
  - The workspace stage now uses the composer dock's measured height as its bottom inset.
  - A `ResizeObserver` updates that inset whenever prompt wrapping, reply context, mode content, or viewport changes alter the dock height.
  - Thread and focused-artifact scroll regions now fill the adjusted stage instead of reserving a second fixed composer offset.
  - The scroll-to-latest control remains 14px above the stage bottom.
- Simulated or deferred behavior: All simulation and production deferrals from r007 remain unchanged.
- Review focus: Confirm the thread scrollbar terminates at the dock divider and continues doing so as the prompt grows and shrinks.
- Open questions: None blocking this round.
- Approval path: Approval keeps the dynamic workspace boundary in r008 and allows the next requested polish item to be applied as another review round in this revision.

## Round 3 — Composer Density And Responsive Alignment

- Date: 2026-07-15
- Phase: Conceptual wireframe polish.
- Revision: `r008-workspace-polish`.
- Feedback applied: Remove the composer caption and its styles, reduce composer dock top padding to 12px, and stop Response Artifacts from shifting left when the viewport crosses below 992px.
- Changed behavior: None.
- Visual changes:
  - Removed the `composer-caption` element and its complete CSS rule.
  - Standardized the composer dock to 12px vertical padding.
  - Removed the 18px transcript-padding override at the 992px overlay breakpoint so ordinary assistant responses and Response Artifacts retain the same centered 760px transcript origin.
  - Kept the compact 18px transcript padding at 620px and below, where the transcript is narrower than 760px and both response types naturally share the same left edge.
- Simulated or deferred behavior: All simulation and production deferrals from r007 remain unchanged.
- Review focus: Composer density and Response Artifact alignment immediately above and below the 992px breakpoint.
- Open questions: None blocking this round.
- Approval path: Approval keeps these polish changes in r008 and allows the next requested polish item to be applied as another review round in this revision.

## Round 4 — Response Artifact Scroll Regions

- Date: 2026-07-15
- Phase: Conceptual wireframe polish.
- Revision: `r008-workspace-polish`.
- Feedback applied: Move Response Artifact scrolling off `.artifact-surface` so scrollbars do not overlap `.artifact__header`, remove unnecessary sticky headers, and audit the shared surface child classes.
- Changed behavior:
  - Browser, File, Folder, and Terminal response cards now share a `header → artifact body → optional footer` structure.
  - Only `.artifact__body` scrolls; `.artifact-surface` clips and composes its children while headers and footers remain static.
  - Dynamic artifact builders reuse one `artifactBodyMarkup()` helper, including folder-to-file transitions and newly submitted artifacts.
- CSS cleanup:
  - Removed sticky header/footer selectors, nested preview/output scrolling, the redundant Folder list height override, and orphaned `.artifact__icon` rules.
  - Consolidated overflow containment and scrollbar gutter behavior on `.artifact__body`.
- Simulated or deferred behavior: All simulation and production deferrals from r007 remain unchanged.
- Review focus: Scroll long artifact content while confirming its header actions and footer status remain unobstructed.
- Open questions: None blocking this round.
- Approval path: Approval keeps the shared artifact body pattern in r008 and allows further polish rounds in this revision.

## Round 5 — Reply Routing And Message References

- Date: 2026-07-15
- Phase: Conceptual wireframe polish.
- Revision: `r008-workspace-polish`.
- Feedback applied:
  - Replying to a Terminal artifact must produce a Chat response rather than creating another Terminal artifact.
  - Replying to an ordinary user or assistant message must identify the quoted context by its first few words rather than `You` or `GPT-5`.
- Changed behavior:
  - Terminal replies now inspect the referenced command state and return a contextual assistant message. The shared terminal session and transcript artifacts remain unchanged.
  - Submitted reply references use `Replying to: <short excerpt>`, capped at eight words with an ellipsis when needed.
  - Browser and File replies continue updating their original artifacts in place and then return a Chat response.
- Simulated or deferred behavior: Terminal status and folder-size answers remain illustrative; no shell command or process inspection occurs.
- Review focus: Reply once to a Terminal artifact and once to an ordinary chat bubble, then confirm the resulting user reference and assistant response type.
- Open questions: None blocking this round.
- Approval path: Approval keeps reply interactions conversational and allows further polish rounds in r008.

## Round 6 — Mode-Specific Controls And Artifact Identity

- Date: 2026-07-16
- Phase: Conceptual wireframe polish.
- Revision: `r008-workspace-polish`.
- Feedback applied:
  - Remove Model and Approval from Files mode.
  - Remove Model, Approval, and Branch from Browser and Terminal modes.
  - Replace `GPT-5` with `C4OS` as the responder shown above every Response Artifact.
- Changed behavior:
  - Composer option controls now declare their semantic role with `data-composer-control`; CSS derives visibility from the active `data-mode`.
  - Chat retains Model, Approval, and Branch. Files retains Branch. Browser and Terminal retain only their mode selector in the toolbar start group.
  - Static and dynamically created Browser, File, Folder, and Terminal artifacts display `C4OS`; ordinary Chat responses continue displaying the selected model.
- Simulated or deferred behavior: Control visibility is conceptual and does not change the existing simulated operations behind each mode.
- Review focus: Switch through all four composer modes and compare the toolbar controls, then inspect both seeded and newly created Response Artifact identities.
- Open questions: None blocking this round.
- Approval path: Approval keeps the simplified mode toolbars and the dedicated C4OS artifact identity in r008.

## Round 7 — Extensible Artifact Shells

- Date: 2026-07-16
- Phase: Conceptual wireframe polish and implementation cleanup.
- Revision: `r008-workspace-polish`.
- Feedback applied: Organize artifact markup, classes, styles, and render helpers so future providers such as GitHub, Figma, Canvas, Airbnb, or Kayak can supply their own bodies without recreating the surrounding response, focus, or compact left-pane treatments.
- Shared shell contract:
  - `artifact-shell--response` owns transcript placement, the C4OS preamble, the framed provider body, and contextual Copy/Reply actions.
  - `artifact-shell--focus` owns the full center-workspace region and shared focus controls while allowing a provider-specific body.
  - `artifact-shell--pane` is a context modifier applied to the existing response shell when the real thread moves into the left Chat pane; it does not introduce a third body implementation.
  - `artifact-provider--<type>` is the only provider-specific root hook. Browser, File, Folder, and Terminal retain their distinct bodies beneath that hook.
- JavaScript cleanup:
  - Centralized response creation in `createResponseArtifactNode()` and shared shell slots in `responseArtifactMarkup()`.
  - Centralized framed response anatomy in `artifactFrameMarkup()` and focused anatomy in `focusArtifactMarkup()`.
  - Added a provider capability registry. Expansion is opt-in per provider; unknown providers safely default to non-expandable and receive a generic icon.
  - Thread context now explicitly changes between `response` and `pane`, updating every existing and newly created artifact consistently.
- CSS cleanup: Replaced historical surface/viewer class families with the shared `artifact-shell`, `artifact-frame`, and `artifact-provider` contracts. Response-body scrolling remains isolated to `artifact-frame__body`; focus and pane modifiers no longer depend on provider-specific DOM ancestry.
- Simulated or deferred behavior: No third-party provider body is implemented in this round. Provider SDKs, authentication, network behavior, and provider-specific focus interactions remain deferred.
- Review focus: Confirm all existing providers retain their behavior in response, focused, and compact left-pane contexts, including dynamic creation and the sub-992px layout.
- Open questions: None blocking this shell provision.
- Approval path: Future providers declare capabilities, supply response/focus body markup, and add only the provider-specific styles they require.

## Round 8 — Agent Response Streaming

- Date: 2026-07-16
- Phase: Conceptual wireframe polish.
- Revision: `r008-workspace-polish`.
- Feedback applied: Make newly generated thinking, working, and response content arrive with a fast typing effect similar to an AI coding harness.
- Changed behavior:
  - New Chat responses open their work disclosure, type the thinking and working states, collapse to the final elapsed label, and then stream the response bubble.
  - New Response Artifacts use the same work sequence, then reveal their C4OS identity, provider frame, and actions as one completed result.
  - One shared generation queue prevents overlapping response animations, and streaming follows the transcript bottom in both center and compact left-pane contexts.
  - Seeded history remains static on reload. Reduced-motion preferences skip the animation while preserving the final content and state.
- Simulated or deferred behavior: Text is streamed from local fixture responses; no model-token or backend event stream is connected.
- Review focus: Submit a Chat prompt and create a Browser artifact, then compare the active work state with the completed response in both transcript placements.
- Open questions: None blocking this polish round.
- Approval path: Approval keeps the fast shared typing sequence and allows the next product phase to begin.

## Round 9 — Chat Attachments And App Dropzone

- Date: 2026-07-16
- Phase: Conceptual wireframe polish.
- Revision: `r008-workspace-polish`.
- Feedback applied: Make the Chat paperclip accept multiple files, show draft files above the input, preview images, truncate long filenames, and use the whole app as a Chat-only drop target.
- Changed behavior:
  - The paperclip opens one native file input with `multiple`; picker and drop interactions share the same draft attachment state.
  - Draft files appear in a horizontally scrollable tray above the Chat input. Image files use object-URL thumbnails; other files use a generic file icon. Every item shows a compact size, truncates its visible filename, preserves the full name in a tooltip, and can be removed independently.
  - Chat can submit files with or without prompt text. Submitted user bubbles retain image previews and file metadata, while the composer clears for the next draft.
  - Dragging files over any part of the app in Chat mode shows one full-app drop indicator. Other composer modes neither advertise nor accept the drop.
  - The existing composer resize observer continues owning the live workspace boundary as the attachment tray appears or disappears.
- Simulated or deferred behavior: Files remain local browser objects for the life of the page. Backend upload, persistence, malware scanning, content extraction, and attachment limits remain deferred.
- Review focus: Add several mixed files, inspect an image preview and truncated long name, remove one item, submit an attachment-only message, and compare drop behavior between Chat and another mode.
- Open questions: None blocking this round.

## Round 10 — Attachment References And Metadata

- Date: 2026-07-16
- Phase: Conceptual wireframe polish.
- Revision: `r008-workspace-polish`.
- Feedback applied: Number attachments for natural-language references and show the file extension beside each compact file size.
- Changed behavior:
  - Draft attachments use a visible one-based number that follows their current order and closes gaps after removal.
  - Submitted attachment previews retain their final one-based numbering so a follow-up can refer to a specific file.
  - Draft and submitted metadata use the shared `<EXT> · <size>` treatment, such as `JSON · 2 KB`, with a MIME-derived fallback when a filename has no extension.
- Simulated or deferred behavior: Attachment numbers are visual reference labels; automatic resolution of phrases such as “use attachment 2” remains deferred to the future agent integration.
- Review focus: Add mixed file types, remove a middle item, confirm the remaining draft numbers reflow, then submit and inspect the retained order and extension metadata.
- Open questions: None blocking this round.

## Round 11 — Newest-First Attachment Presentation

- Date: 2026-07-16
- Phase: Conceptual wireframe polish.
- Revision: `r008-workspace-polish`.
- Feedback applied: Present the latest attachment first in both the composer and submitted messages, and visually separate submitted files from the user prompt bubble.
- Changed behavior:
  - One shared ordering helper renders attachment references newest first without mutating their selection order, producing labels such as `4, 3, 2, 1`.
  - Submitted files occupy a dedicated, softly framed attachment group above the ordinary user prompt bubble.
  - Attachment-only submissions render the file group without creating an empty text bubble.
  - Copy remains safe for text-plus-file and attachment-only user messages after the message anatomy change.
- Simulated or deferred behavior: The prototype does not yet provide per-attachment context menus or reorder controls.
- Review focus: Add four files, compare the `4, 3, 2, 1` order before and after submission, and inspect a text-plus-file message in both center and left-pane thread contexts.
- Open questions: None blocking this round.

## Round 12 — WYSIWYG Markdown Chat

- Date: 2026-07-16
- Phase: Conceptual wireframe polish.
- Revision: `r008-workspace-polish`.
- Feedback applied: Make the Chat and Reply inputs WYSIWYG Markdown editors without a formatting toolbar, keep Enter to send and Shift+Enter for a line break, and render both user and assistant Chat bubbles as Markdown.
- Changed behavior:
  - Chat and Reply now use shared `contenteditable` WYSIWYG surfaces. Native rich-text shortcuts and pasted Markdown render in place, while submission serializes the editor DOM back to Markdown source.
  - User and assistant bubbles share one safe Markdown renderer for headings, emphasis, links, blockquotes, lists, task lists, tables, inline code, and fenced code. Raw HTML is escaped and unsafe link protocols resolve to a non-navigating hash.
  - Copy reads the preserved Markdown source, Reply uses a plain-text excerpt, and fenced code includes a contextual Copy control.
  - Assistant response streaming progressively re-renders incomplete Markdown chunks, then leaves the same clean final markup. Thinking and working disclosures continue using plain text.
  - Shift+Enter inserts an explicit editor line break; Enter submits after composition completes. The composer’s existing live-height measurement continues to move the workspace boundary as rich content grows.
- Simulated or deferred behavior: The local renderer intentionally covers the wireframe’s common Markdown subset rather than every CommonMark extension. Toolbar controls, slash commands, collaborative selection, and backend Markdown parsing remain deferred.
- Review focus: Paste a mixed Markdown sample, confirm the rich editor appearance and Markdown source on submission, compare user and streaming assistant markup, use Shift+Enter, and inspect code Copy plus unsafe-link handling.
- Open questions: None blocking this round.
- Approval path: Approval keeps the toolbar-free WYSIWYG editor and shared safe Markdown runtime in r008.

## Round 13 — Source-Preserving Markdown Editor

- Date: 2026-07-16
- Phase: Conceptual wireframe polish.
- Revision: `r008-workspace-polish`.
- Feedback applied: Continue after the stalled Markdown pass; fix heading, unordered-list, ordered-list, fenced-code, and inline formatting loss; accept a semi-raw Markdown/WYSIWYG combination; convert a selected label plus pasted URL into a Markdown link; and make raw URLs visually link-like.
- Changed behavior:
  - Chat and Reply now use a hybrid highlighter. Markdown markers remain visible, their effects appear in place, and the exact source text remains the submission contract.
  - Re-highlighting after each edit prevents the browser from silently replacing Markdown list prefixes with browser-owned rich-list markup.
  - The renderer consistently handles headings, unordered and ordered lists, emphasis, fenced code, explicit links, and standalone `http`, `https`, or `www` addresses.
  - Pasting a URL over selected text replaces the selection with `[label](url)` source. Pasting a URL without a selection keeps the raw address and highlights it in blue; submitted raw addresses become safe links.
- Simulated or deferred behavior: The editor provides source-aware formatting rather than a full document-model editor. It does not include a formatting toolbar, collaborative cursors, or arbitrary nested CommonMark block editing.
- Review focus: Type the original failing sample line by line, confirm markers stay visible and formatted, submit it, then test selected-text URL paste and a standalone raw URL.
- Open questions: None blocking this round.
- Approval path: Approval keeps the hybrid source-preserving editor in r008 and closes this Markdown interaction round.

## Round 14 — CodeMirror Markdown Composer

- Date: 2026-07-16
- Phase: Conceptual wireframe polish.
- Revision: `r008-workspace-polish`.
- Feedback applied: Remove the stalled custom editor implementation, do not add non-Markdown underline behavior, and replace it with a stable source editor inspired by established Markdown editors.
- Changed behavior:
  - Removed the contenteditable serializer, selection walker, refresh-on-input formatter, prefix interception, and custom paste reconstruction.
  - Chat and Reply now share one locally bundled CodeMirror 6 editor configured for Markdown source, fast inline syntax effects, native selection and undo history, and no formatting toolbar.
  - Cmd/Ctrl+B and Cmd/Ctrl+I wrap or unwrap selected source. Shift+Enter continues unordered lists, increments ordered lists, continues quotes, or exits an empty marker; Enter sends.
  - Selected text plus a pasted web address becomes Markdown link source. Standalone web addresses remain source text, receive link styling, and autolink in the submitted bubble.
  - User and assistant bubbles now use locally bundled `markdown-it` with raw HTML disabled, linkification, task-list support, and the established fenced-code Copy surface.
- Implementation boundary: `markdown-source.js` is the maintainable editor/renderer source. `markdown.js` is its generated, document-relative browser bundle so the wireframe continues working from `file://` without a runtime CDN.
- Simulated or deferred behavior: Underline, a formatting toolbar, collaboration, slash commands, tables inside the composer, and a fully WYSIWYG document model remain out of scope.
- Review focus: Re-run the previously failing heading, emphasis, list, ordered-list, fenced-code, link-paste, undo, Enter, and Shift+Enter cases in Chat and Reply.
- Open questions: None blocking this round.
- Approval path: Approval keeps the source-native CodeMirror editor and `markdown-it` response rendering in r008.
