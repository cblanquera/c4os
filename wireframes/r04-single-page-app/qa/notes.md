# QA Notes

Date: 2026-06-20

## r04 Single Page App Conversion

Source revision:

- `wireframes/r03-functional-frontend-architecture/`

Changes made:

- Created `r04-single-page-app` as a new sibling artifact.
- Kept only one HTML entry point: `index.html`.
- Removed the starting review menu from the default app flow.
- Changed the default entry route to App Start.
- Preserved r03's screen data, render functions, settings surfaces, and visual CSS.
- Added hash-based routes for all generated internal links.
- Added a client-side router that intercepts review navigation, swaps rendered screens in place, and supports direct hash entry.

Route behavior:

- App start: `index.html#app-start`
- New session: `index.html#new-session`
- Chat session: `index.html#chat-session`
- Providers popover: `index.html#providers-popover`
- Models popover: `index.html#models-popover`
- File explorer: `index.html#file-explorer`
- File editor: `index.html#file-editor`
- Terminal: `index.html#terminal`
- Settings providers: `index.html#settings-providers`
- Add provider: `index.html#settings-add-provider`
- Settings models: `index.html#settings-models`
- Settings configuration: `index.html#settings-configuration`
- Settings plugins: `index.html#settings-plugins`
- Settings skills: `index.html#settings-skills`
- Settings MCP servers: `index.html#settings-mcp`

Verification:

- `node --check wireframes/r04-single-page-app/script.js` passed.
- `find wireframes/r04-single-page-app -maxdepth 1 -type f -name '*.html'` returns only `index.html`.
- `git diff --check -- wireframes/r04-single-page-app` passed.
- Static server returned `200 OK` for `/`, `/script.js`, and `/styles.css` at `http://127.0.0.1:4184/`.
- After menu removal, Playwright loaded `http://127.0.0.1:4184/` directly into App Start with the `Open a folder to start working` heading and no review menu.
- Playwright loaded the app with no console errors after adding the inline favicon.
- Direct hashes remain supported for screen review.
- Playwright direct hash loads for `#file-editor` and `#settings-mcp` rendered the expected screens.
- Unused style cleanup removed the old hub/menu CSS and dead emitted class names: `browser-tool`, `marketplace-modal`, `mcp-modal`, `skill-modal`, and `is-on`.
- Static class mismatch scan now only reports expected template fragments from dynamic class names.

## r03 Source QA Notes

Preview server:

- `http://127.0.0.1:4183/`

## Peg Diff Pass 01

Compared against:

- `plans/product-interface.md`
- `wireframes/screens.md`
- `plans/pegs/C4OS-App-Start-Wireframe.png`
- `plans/pegs/C4OS-New-Session-Wireframe.png`
- `plans/pegs/C4OS-Chat-Session-Wireframe.png`
- `plans/pegs/C4OS-Providers-Popover-Wireframe.png`
- `plans/pegs/C4OS-Models-Popover-Wireframe.png`
- `plans/pegs/C4OS-File-Explorer-Right-Wireframe.png`
- `plans/pegs/C4OS-File-Editor-Right-Wireframe.png`
- `plans/pegs/C4OS-Terminal-Right-Wireframe.png`
- `plans/pegs/C4OS-Settings-Providers-Wireframe.png`
- `plans/pegs/C4OS-Settings-Add-Provider-Wireframe.png`
- `plans/pegs/C4OS-Settings-Models-Wireframe.png`
- `plans/pegs/C4OS-Settings-Configuration-Wireframe.png`

Concrete diffs identified before editing:

- App start: r03 used a stronger trust-spine treatment and different body copy; peg is neutral black/white and uses the folder-first scoping paragraph.
- Shell proportions: r03 panels were more fluid/stylized than peg proportions; peg shows a wide fixed left rail and fixed right tool panel.
- Center workspace: r03 background grid and green state accents diverged from the peg's plain white center canvas.
- Composer: r03 composer was narrower and vertically lighter than peg; peg composer has a larger prompt area, taller control rows, and branch-only context row.
- Right panel: r03 right tabs exposed text labels; peg tabs are icon-only. Browser address used `3000`; peg uses `13000`.
- Chat: r03 message/activity cards were smaller and more colored than the peg's large grayscale message blocks.
- File states: r03 file active state used green accent; peg uses neutral grayscale selection.
- Terminal: r03 terminal used dark code styling; peg terminal is grayscale with a read-only bottom panel.
- Settings: r03 settings rail was narrower and headings larger than peg proportions.

Changes made:

- Shifted r03 palette back toward neutral grayscale for peg alignment.
- Tuned shell panel variables toward peg-like left/right proportions.
- Removed decorative center grid and reduced trust-spine intensity to resize-handle treatment.
- Enlarged composer max width and row heights to better match the new-session peg.
- Changed composer context icon to branch and kept only `main` in the context strip.
- Made right tool tabs visually icon-only while preserving accessible labels.
- Changed browser address to `http://127.0.0.1:13000`.
- Reduced green active-state treatment in sidebar, files, and tool tabs.
- Changed code and terminal surfaces back to light grayscale.
- Increased chat card heights and activity rows.
- Increased settings rail width and normalized settings heading/list proportions.

Responsive review:

- CSS now keeps desktop app proportions for broad desktop widths and compresses at `1180px`.
- At `940px`, the artifact preserves a narrow desktop shell with horizontal overflow instead of stacking the app like a mobile website.
- Final Chrome headless screenshots captured:
  - `qa/screenshots/new-session-1440-final.png`
  - `qa/screenshots/new-session-1280-final.png`
  - `qa/screenshots/new-session-1180-final.png`
  - `qa/screenshots/new-session-940-final.png`

## Peg Diff Pass 02

Additional rendered diffs identified from Chrome headless screenshots:

- `new-session`: composer was too wide/tall versus peg; branch row was too chip-like; right resize handle was too noisy.
- `chat-session`: activity cards were too narrow; docked follow-up composer was clipped at the bottom; follow-up composer showed branch context row that the chat peg does not show.
- `models-popover`: popover overlapped the screen title instead of anchoring near the model chip.
- `terminal`: terminal bottom panel was dark and too stylized compared with peg.
- `940px`: app regions stacked vertically and sliced the shell like a webpage instead of preserving a narrow desktop app shell.

Changes made:

- Reduced composer width/height and made it responsive with `width: min(700px, 100%)`.
- Preserved visible `main` in the branch row while simplifying context-row styling.
- Lowered model/provider popovers to anchor nearer the model chip.
- Made activity cards peg-width and reduced chat vertical density so the docked composer fits.
- Hid the docked chat composer context row, matching the chat peg.
- Switched narrow desktop behavior to preserve the 3-panel shell with horizontal overflow at `940px`.
- Kept right tool tabs icon-only and accessible.

Rendered evidence captured:

- `qa/screenshots/app-start-1440.png`
- `qa/screenshots/new-session-1440-final.png`
- `qa/screenshots/chat-session-1440-pass4.png`
- `qa/screenshots/providers-popover-1440-pass2.png`
- `qa/screenshots/models-popover-1440-pass2.png`
- `qa/screenshots/file-explorer-1440.png`
- `qa/screenshots/file-editor-1440.png`
- `qa/screenshots/terminal-1440.png`
- `qa/screenshots/settings-providers-1440.png`
- `qa/screenshots/settings-add-provider-1440.png`
- `qa/screenshots/settings-models-1440.png`
- `qa/screenshots/settings-configuration-1440.png`

Remaining visual differences:

- Right-panel variant pages (`file-explorer`, `file-editor`, `terminal`) keep the shared center new-session shell while changing the right panel. This matches the purpose of those peg-backed right-panel states, but the review focus should stay on the right panel for those pages.
- r03 uses real text blocks where some pegs use placeholder skeleton bars, so copy density differs slightly while preserving layout behavior.
- Exact icon geometry differs because the draft uses inline SVG paths rather than the final Lucide package.

Route checks:

- `200 index.html`
- `200 app-start.html`
- `200 new-session.html`
- `200 chat-session.html`
- `200 providers-popover.html`
- `200 models-popover.html`
- `200 file-explorer.html`
- `200 file-editor.html`
- `200 terminal.html`
- `200 settings-providers.html`
- `200 settings-add-provider.html`
- `200 settings-models.html`
- `200 settings-configuration.html`

Code checks:

- `node --check wireframes/r03-functional-frontend-architecture/script.js` passed.
- Static scan found no root-relative generated links.
- Static scan found no `transition: all`.
- Static scan found no disabled browser zoom settings.
- Static scan found no inline `onclick` handlers.
- Static scan found no `innerHTML` usage.
- Static scan did report `.map()` calls, but they are bounded review-data arrays well below the guideline's large-list virtualization threshold.
- Route files use document-relative `./styles.css` and `./script.js`.

Guideline-oriented checks:

- Icon-only buttons are generated with `aria-label`.
- Form fields are generated with explicit labels, ids, names, autocomplete attributes, and suitable input types.
- The contenteditable prompt has `role="textbox"`, `aria-label`, and `aria-multiline`.
- Focus-visible styles, reduced-motion handling, skip links, and text overflow handling are included.

Browser automation:

- Playwright screenshots remain unavailable because bundled Chromium is not installed and system Chrome exited under Playwright automation in the earlier pass.
- Chrome direct headless screenshots succeeded and are saved under `qa/screenshots/`.

## Feedback Fix Pass 03

Requested fixes:

- Models popover heading left padding had an unwanted blank offset before the back chevron.
- Show More / Show Less needed to sit under the full message text, including expanded text.
- Chat session names in the sidebar needed to align flush under the project name.
- Message thread needed to scroll, and the chat prompt needed to remain reachable/visible.
- Add Provider hover state was too light.

Changes made:

- Split popover header variants into `popover-title` and `popover-backbar`, then removed the double left-padding from the model back header.
- Reordered expanded message DOM so extra text renders before the Show Less control.
- Adjusted `.session-row` indentation so nested session labels align with the project label text.
- Changed `.thread-view` to scroll as a full chat surface and made `.thread-list` overflow visible so the docked composer can be reached.
- Kept the docked chat composer visible by reducing its prompt height and hiding the branch context row in thread mode.
- Added a stronger `.primary:hover` state so Add Provider stays high contrast on hover.

Rendered evidence captured:

- `qa/screenshots/models-popover-feedback-fixes.png`
- `qa/screenshots/chat-session-feedback-fixes.png`
- `qa/screenshots/new-session-feedback-fixes.png`
- `qa/screenshots/settings-providers-feedback-fixes.png`

Verification:

- `node --check wireframes/r03-functional-frontend-architecture/script.js` passed.
- All 13 routes returned `200`.
- Static guideline scan found no root-relative generated links, disabled zoom, `transition: all`, inline `onclick`, `innerHTML`, or paste blocking.
