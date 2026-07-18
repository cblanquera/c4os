# Wireframe Decision Ledger

## How To Read This Ledger

Each entry records the durable effect of a revision or review round. Later entries supersede conflicting earlier ones. QA-only observations confirm the source or rendered behavior; browser limitations are historical and do not weaken later r012 verification.

## r001 — AI Chat Shell

- R1 established a resizable three-panel desktop shell, centered chat, fixed composer, blank side panels, prompt controls, narrow overlays, and local simulations.
- R2 removed `You`, used model identity for assistant responses, introduced assistant bubbles and `Worked for <duration>`, and ordered Attach → model → approval → branch.
- R3 replaced boxed work summaries with an unboxed user-facing activity stream; private reasoning is excluded.

## r002 — Response Artifact Modes

- R1 added persistent Chat/Files/Browser/Terminal modes, structured artifacts, contextual Copy/Reply, reply restoration, explicit expansion, and provider-specific update behavior.
- R2 moved thinking above model/content, centered artifact identity, moved Copy/Reply below cards, and kept Expand in headers.
- R3 unified ordinary and artifact response preambles with thinking then identity then content.
- R4 aligned all response rows flush left and constrained artifact reading width.
- R5 consolidated response radius, geometry, identity alignment, and contextual-action CSS.

## r003 — Composer Mode Popover

- R1 replaced persistent mode tabs with a compact upward popover left of Attach.
- R2 consolidated Browser navigation/address/Expand, added line-numbered resizable File editing, made Files Browse icon-only, and removed repeated commands from Terminal output.
- R3 moved type icons into the assistant identity row and aligned boxed Browser/Terminal prefixes.
- R4 repaired the expanded Browser navigation-header pattern. Its right-pane destination was later superseded, but the toolbar pattern remains.

## r004 — Artifact Focus Swap

- R1 removed the right pane/tabs and made Browser/File Expand replace center Chat while the real thread moved into a draggable floating frame; composer stayed fixed. Terminal Expand was temporarily removed.

## r005 — Left Chat Pane

- R1 replaced the floating frame with a bottom pane in the left panel, set 40% default/60% maximum height, preserved collapsed state, and repaired repeated width resizing.
- R2 locked composer to Chat during focus, made pane artifacts read-oriented, filled center with focused File, and added direct focused Close controls.

## r006 — File Explorer And Artifact Refinement

- R1 added File/Folder selection, Explorer artifacts, breadcrumbs, focused line numbers, long-file scrolling, and non-functional Detach Chat.
- R2 added path-specific folders and file opening inline/focused.
- R3 made opened files editable and unified Explorer render/state paths.
- R4 moved file editing to contenteditable/source-preserving surfaces, synchronized line numbers, removed focused Explorer footer, and restored narrow panel resizing.
- R5 bounded all inline artifacts with internal scroll, changed overlay breakpoint to 992px, aligned Browser globe icons, and kept inline breadcrumb navigation inline.
- R6 clarified Discard versus Close, removed File view/edit footers, and compacted left-pane message text.
- R7 removed File edit controls from contextual-pane cards.
- R8 ensured every user prompt has Copy/Reply.
- R9 added one shared scroll-to-latest control with a 24px threshold.
- R10 made focused File view/edit scroll internally while header/composer remain fixed.

## r007 — Terminal Session Workspace

- R1 restored Expand to all Terminal cards, introduced one persistent `shell-1`, running-process Stop/`Ctrl+C`, stdin, and a terminal-local prompt while global composer stays Chat.
- R2 made the expanded view one continuous selected-command/process body; history remains in transcript cards.

## r008 — Workspace Polish

- R1 added a subtle top divider to the composer dock.
- R2 made the workspace stage end at the live multiline composer edge.
- R3 removed disclaimer copy, reduced dock padding, and preserved shared transcript alignment below overlay breakpoint.
- R4 standardized artifact shells with only bodies scrolling.
- R5 made Reply always Chat and changed message reference labels to excerpts.
- R6 reduced non-Chat controls and standardized artifact responder identity as C4OS.
- R7 formalized response/focus/pane shells and provider-declared expansion.
- R8 added generated response streaming with seeded history static and reduced-motion support.
- R9 added multi-file picker, removable/image attachments, attachment-only send, and app-wide Chat dropzone.
- R10 added stable attachment reference numbers and metadata.
- R11 displayed newest attachments first while preserving original numbers.
- R12 introduced rendered Markdown chat.
- R13 preserved Markdown source for Copy/Reply/editing.
- R14 finalized the toolbar-free source editor, Markdown shortcuts, paste-link behavior, and undo.

## r009 — Settings

- R1 established the Settings shell and Providers/Models/Runtimes/Configuration/Plugins/Skills/MCP destinations.
- R2 aligned the navigation hierarchy; R3 compacted C4OS/menu scale.
- R4 added provider profiles and shared Add/Edit; R5 simplified rows; R6 repaired compatible-provider conditional fields.
- R7 added model search/provider filter/availability; R8 clarified visible-result bulk action.
- R9 added runtime draft/save; R10 aligned runtime radio treatment.
- R11 simplified Configuration; R12 removed Default model.
- R13 added nine-group/71-identity Advanced Policies; R14 kept Configuration selected and removed redundant back link; R15 aligned Save.
- R16 added plugin tabs, marketplace source form, uninstall, and detail state; R17 added detail Install; R18 added identity and policy links.
- R19 removed Configuration eyebrow; R20 removed Plugins eyebrow.
- R21 added searchable Skills and detail management; R22 aligned its close action.
- R23 added MCP STDIO/HTTP management and repeatable fields.
- R24 corrected all Settings and Advanced Policies into the existing SPA, preserving workspace state. This is the binding architecture.

## r010 — Onboarding And Start

- R1 copied the full SPA and added first-provider onboarding plus provider-present start in the same document.
- R2 refined launch-screen annotations/copy/layout.
- R3 required a new Terminal response card for each command.
- R4 preserved File edit state across focus changes.
- R5 preserved File Explorer navigation/open/edit state when restoring inline.

## r011 — Workspace Projects

- R1 added search-first projects/sessions, native folder selection, project menus, hover actions, session remove, sorting, and six seeded sessions across three projects.
- R2 represented missing path with lighter italic text and allowed the list to fill remaining panel height.
- R3 deferred session creation until first valid submission and titled from the first prompt.
- R4 reduced the workspace header to title only.
- R5 added attachment-only promotion/title fallback, clean New chat transition, removal fallbacks, accessible rename structure, and reliable clipboard feedback.
- R6 top-aligned stacked start-card icons with copy; R7 removed excess forced height.

## r012 — Cleanup And Final Baseline

- R1 changed source organization only: consistent formatting, alphabetized CSS, ownership comments, component class naming, and section/JSDoc comments while preserving all r011 behavior.
- Browser QA verified project More/Rename/New chat, Providers, Plugins, onboarding, and all fourteen launcher destinations with no console errors.

## Superseded Directions

- Blank side panels → left project/session navigation.
- Persistent mode tabs → compact mode popover.
- Right artifact pane/tabs → single center artifact focus.
- Floating Chat frame → contextual bottom pane in left panel.
- Terminal without Expand → every Terminal card expandable in one persistent session.
- Grayscale-only product styling → platform-native light/dark styling; grayscale remains structural evidence.
- Separate Settings pages/files → one stateful application shell.
