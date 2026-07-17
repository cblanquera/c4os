# Visual QA — r008

## Round 1 — Composer Dock Divider

- Review target: `http://127.0.0.1:4180/index.html#terminal`.
- Computed `.composer-dock` border: `1px solid rgb(195, 195, 195)`, matching `--gray-300`.
- The divider spans the center workspace above the dock in the normal Terminal transcript state.
- The divider remains present after expanding a Terminal artifact while the fixed AI composer returns to Chat mode.
- Browser inspection found zero horizontal document overflow in normal and expanded states.

## Round 2 — Dynamic Composer Boundary

- Review target: `http://127.0.0.1:4180/index.html#chat`.
- At the default one-line prompt height, the composer dock measured 157px and the workspace stage, thread viewport, and dock top all met at pixel 563 with a zero-pixel delta.
- With a five-line prompt, the dock grew to 241px and its top moved to pixel 479; the workspace stage and thread viewport updated to the same pixel with a zero-pixel delta.
- Shrinking the prompt restored the 157px dock and pixel-563 boundary without reload.
- The runtime `--composer-dock-height` value tracked the measured dock height in each state.
- Browser inspection found zero horizontal document overflow after multiline expansion.

## Round 3 — Composer Density And Responsive Alignment

- Review target: `http://127.0.0.1:4180/index.html#terminal`.
- The rendered page contains zero `.composer-caption` elements, and the caption CSS selector is absent from the stylesheet.
- Computed composer dock padding is 12px on both top and bottom; the dynamic workspace boundary still meets the dock with a zero-pixel delta.
- At the desktop review viewport, the ordinary assistant response and Browser Response Artifact shared the same left coordinate with a zero-pixel delta.
- CSS media-rule inspection confirmed that the 991.98px breakpoint no longer overrides `.thread` gutters; the compact `.thread` rule exists only in the 620px breakpoint.
- Browser inspection found zero horizontal document overflow, and JavaScript syntax plus whitespace checks passed.

## Round 4 — Response Artifact Scroll Regions

- Review target: `http://127.0.0.1:4180/index.html#chat` and Files mode on the same revision.
- Browser, File, Folder, and Terminal response surfaces each rendered one direct `.artifact__body` child.
- Computed `.artifact-surface` vertical overflow is `hidden`; computed `.artifact__body` vertical overflow is `auto`.
- Artifact headers and footers compute to `position: static`; no sticky response-artifact selectors remain.
- The long Terminal body measured 509px of scroll content within a 352px viewport. Scrolling it to the bottom moved only the body (`scrollTop: 474`); the surface stayed at `scrollTop: 0` and the header top coordinate did not change.
- A dynamically submitted Folder artifact rendered direct children in the shared order: header, body, footer.
- JavaScript syntax, whitespace checks, and the orphan-selector audit passed.

## Round 5 — Reply Routing And Message References

- Review target: `http://127.0.0.1:4180/index.html#chat`.
- Before replying to the running Terminal artifact, the transcript contained one Terminal artifact and one assistant message.
- After asking `is the server still on?`, the transcript still contained one Terminal artifact and gained one assistant Chat response: `Yes. The referenced process is still running in the shared terminal session.`
- The submitted Terminal reference rendered as `Replying to: Terminal · npm run dev`.
- Replying to the new assistant Chat response rendered `Replying to: Yes. The referenced process is still running in…`, confirming that chat references use a short excerpt rather than the model label.
- JavaScript syntax and whitespace checks passed.

## Round 6 — Mode-Specific Controls And Artifact Identity

- Review target: `http://127.0.0.1:4180/index.html#chat`, switching through Files, Browser, and Terminal modes.
- Computed visible option controls by mode:
  - Chat: Model, Approval, Branch.
  - Files: Branch only.
  - Browser: none of Model, Approval, or Branch.
  - Terminal: none of Model, Approval, or Branch.
- All three seeded Response Artifacts displayed `C4OS`; the seeded ordinary Chat response remained `GPT-5`.
- Submitting a new Browser artifact increased the artifact count from three to four and the new artifact displayed `C4OS`.
- Browser inspection found zero horizontal document overflow. JavaScript syntax and whitespace checks passed.

## Round 7 — Extensible Artifact Shell Refactor

- Review target: `http://127.0.0.1:4180/index.html`, exercised in Chat, Browser focus, File focus/edit, and Terminal focus/interruption states.
- Static validation:
  - `node --check` passed for `script.js`.
  - `git diff --check` passed.
  - The historical `.artifact-surface`, `.artifact__*`, `.artifact-actions`, `.artifact-model-row`, `.artifact-thinking`, `.artifact-header-actions`, `.focus-artifact`, and `.pane-view` class families are absent from current HTML, CSS, and JavaScript.
  - HTML Tidy reported no structural errors. Its warnings were limited to intentionally empty, script-populated controls and one standards-valid global ARIA attribute that Tidy labels proprietary.
- Shared response contract: Each seeded Browser, File, and Terminal card rendered exactly one direct `artifact-shell__preamble`, `artifact-frame`, and `artifact-shell__actions` child. Every `artifact-frame__body` computed to `overflow: auto`; no historical shell selectors remained in the live DOM.
- Dynamic creation: Submitting a new Browser artifact increased the card count from three to four. The new card inherited the response shell, frame, action slots, `data-artifact-context="response"`, and `C4OS` identity without provider-specific outer markup.
- Focus and pane transitions:
  - Browser, File, and Terminal Expand each produced one `artifact-shell--focus` with `data-artifact-context="focus"`.
  - The real thread switched to `data-artifact-context="pane"`; all three transcript artifacts gained `artifact-shell--pane`.
  - Closing focus restored `data-artifact-context="response"`, removed every pane modifier, and returned the thread to the center host.
- Provider regressions:
  - Browser focus retained its address and navigation shell.
  - File focus entered contenteditable edit mode with visible fixed line numbers and `overflow: auto`; discard restored read mode and close restored Chat.
  - Terminal focus retained the selected continuous process body. Stop marked the response Interrupted with exit 130 and restored the trailing `$` prompt.
- Responsive review at 800×819:
  - The ordinary assistant response, artifact shell, and artifact frame shared the same 28px left coordinate and 654.71875px width.
  - Browser focus filled the 800px center width with no horizontal document overflow.
  - Expanding while the left panel was closed kept it closed; manually opening the panel exposed the compact pane shell, and clicking the center dismissed the overlay.
- Browser console review found no warnings or errors. The temporary viewport override was reset after review.

## Round 8 — Agent Response Streaming

- Review target: `http://127.0.0.1:4180/index.html#chat` in the center transcript and Browser-focused left Chat pane.
- A newly submitted Chat response immediately exposed `aria-busy="true"`, opened its work disclosure, typed `Thinking…` while the work-summary text was still partial, and kept the final response bubble hidden.
- Completion collapsed the disclosure to `Worked for 12 sec`, removed `aria-busy`, revealed the GPT-5 response bubble, and produced the complete fixture response.
- A newly submitted Browser artifact followed the same active work state with its C4OS identity and provider frame hidden; completion revealed both, retained `https://openai.com`, and collapsed to `Worked for 8 sec`.
- With Browser focused, a new Chat response streamed inside the real thread in the left pane, finished at a zero-pixel distance from the transcript bottom, and kept the scroll-to-bottom control hidden.
- Reloading preserved three static seeded artifacts and the seeded assistant response with zero active stream or busy nodes, confirming that completed history does not replay.
- The page reported zero horizontal document overflow, the browser console contained no warnings or errors, and JavaScript syntax plus whitespace checks passed.

## Round 9 — Chat Attachments And App Dropzone

- Review target: `http://127.0.0.1:4180/index.html#chat`.
- The Chat composer exposes one native `type=file` input with the `multiple` property enabled, connected to the plural `Attach files` paperclip control.
- The initial draft tray and full-app drop indicator are hidden, the Send action is disabled without text or files, and the page has zero horizontal document overflow.
- The workspace stage and composer dock share the same 599px boundary before attachments, confirming the existing live dock measurement remains intact.
- The real paperclip interaction opens the native picker. Browser automation cannot populate the operating-system file dialog, so mixed-file preview, removal, and submitted-message states remain the human-review portion of this round.
- Source-path checks confirmed that picker changes and Chat-only drops call the same attachment-state function, attachment-only sends are enabled, image object URLs are retained through submission, and unused preview URLs are revoked on removal or unload.
- JavaScript syntax and whitespace checks passed.

## Round 10 — Attachment References And Metadata

- Draft and submitted attachment markup share the same one-based numbering and `<EXT> · <size>` metadata helpers.
- Draft numbers derive from the current attachment order, so removing an item closes numbering gaps without changing attachment state.
- Submitted attachments derive and retain their final message order independently of later composer drafts.
- Extension labels prefer the filename suffix, fall back to the MIME subtype, and use `FILE` only when neither is available.
- JavaScript syntax and whitespace checks passed. Refreshing the local `file://` review tab and repopulating the native picker remains the human visual check.

## Round 11 — Newest-First Attachment Presentation

- Draft and submitted attachment renderers use the same non-mutating newest-first entry helper while retaining original one-based reference labels.
- Submitted attachment groups are siblings of the prompt bubble rather than children of it; attachment-only messages omit the empty prompt surface.
- Center, narrow, and left-pane selectors give the attachment group the same maximum alignment boundary as the corresponding user bubble.
- User-message copy handles an attachment group, a text bubble, or both without assuming either region exists.
- The newest-first helper returned `4:four,3:three,2:two,1:one` in its focused ordering check; JavaScript syntax and whitespace checks passed.

## Round 12 — WYSIWYG Markdown Chat

- Headless Chrome loaded the local `file://` wireframe with the Markdown runtime available, both contenteditable editors initialized, five seeded message bodies hydrated, and no runtime exceptions.
- Pasting `### Pasted heading` plus bold Markdown produced an `h3` and `strong` element in the editor, then serialized back to the same Markdown source.
- A mixed heading, bold text, safe link, list, and fenced JavaScript block rendered in the editor and serialized without loss. Enter submitted it, cleared the editor, retained the source on the user bubble, and rendered the expected heading, emphasis, link, list, code surface, and code Copy control.
- Shift+Enter preserved the current draft and serialized `line one\nline two`; it did not add a message. Enter submission remained active after input composition checks.
- The generated assistant bubble exposed `aria-busy="true"` during streaming, completed without the busy state, retained its Markdown source, and rendered the streamed bold lead-in.
- Raw HTML remained visible as escaped text, a `javascript:` Markdown link resolved to `#`, and a normal HTTPS link retained the `https:` protocol.
- The final 1220×900 visual capture showed the rich user Markdown and streamed assistant Markdown aligned with the existing bubble system and fixed composer.
- `node --check` passed for `markdown.js` and `script.js`; `git diff --check` passed.

## Round 13 — Source-Preserving Markdown Editor

- A focused renderer fixture covered the reported heading, bold, italic-with-trailing-space, unordered-list, ordered-list, fenced-code, explicit-link, raw-`https` URL, raw-`www` URL, and alternate-bold cases.
- The message renderer produced the expected semantic `h3`, `strong`, `em`, `ul`, `ol`, fenced-code, and safe anchor markup. Both explicit and standalone StackEdit addresses resolved to HTTPS links.
- The hybrid editor renderer retained the fixture source byte-for-byte after its formatting spans were removed, including visible Markdown markers and the space before the italic closing marker.
- Unsafe explicit link protocols still resolve to `#`. Raw URL support is limited to web addresses and does not broaden the existing safe-protocol policy.
- A `beforeinput` guard now owns spaces after heading, quote, list, and ordered-list prefixes so the browser cannot replace Markdown source with native rich-list DOM.
- Selected-text URL paste is implemented through source offsets: the selected source becomes `[label](url)`, while a collapsed selection keeps the raw address for blue source highlighting and post-send autolinking.
- `node --check` passed for `markdown.js` and `script.js`; `git diff --check` passed. The in-app browser could not reload the local `file://` artifact under its URL policy, so interaction and visual confirmation remain the explicit review target for this round.

## Round 14 — CodeMirror Markdown Composer

- Headless Chrome loaded the local `file://` artifact with two CodeMirror instances and no page or console errors.
- The old contenteditable-specific selection, serializer, refresh, prefix, and paste paths are absent from `script.js`; Chat and Reply initialize through one shared editor runtime.
- Cmd/Ctrl+B produced `**Bold and italic**`; undo restored the source and repeating the shortcut reapplied the markers.
- Shift+Enter transformed `- first` into two unordered rows, exited the empty list marker, then incremented `1. one` to `2. two`.
- Enter fired the expected form submission, preserved the canonical source on the user message, cleared the composer, and rendered one `h2`, one `strong`, one unordered list, and one ordered list.
- The hidden Reply editor reused the same runtime: opening Reply, setting `**Reply body**`, and pressing Enter produced a second submission with the Markdown source and quoted message reference intact.
- Pasting `https://stackedit.io/` over selected `Link Test` produced `[Link Test](https://stackedit.io/)`; a standalone address received one raw-URL editor decoration.
- Renderer fixtures confirmed tables receive the responsive `markdown-table` wrapper and task items receive the plugin's shared `task-list-item` class.
- The 1168×964 editor-state and submitted-message captures retained existing transcript alignment, fixed composer layout, and Markdown effects without horizontal overflow.
- `markdown-source.js`, generated `markdown.js`, and `script.js` passed `node --check`; `git diff --check` passed.
