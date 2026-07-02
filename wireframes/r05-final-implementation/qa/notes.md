# QA Notes

Date: 2026-07-02

## Batch 1 Scope Check

- Artifact is limited to frontend wireframe files under `wireframes/r05-final-implementation/`.
- No `.agents` spec files, `wireframes/screens.md`, or `wireframes/ui-handoff-spec.md` were updated before approval.
- Links in `index.html` and generated route links are document-relative or hash-only.
- The artifact uses grayscale HTML/CSS/JS only.
- The local `.agents/workflows/feedback-loop.md` file requested in the goal does not exist in this checkout; the `chrisai-designing` feedback-loop workflow was used for review-round structure.

## Verification

- `node --check wireframes/r05-final-implementation/script.js` passed.
- `git diff --check -- wireframes/r05-final-implementation` passed.
- Static scan found no root-relative `href`, `src`, or CSS `url()` paths.
- Static server returned `200 OK` for `/`, `/script.js`, and `/styles.css` at `http://127.0.0.1:4185/`.
- Browser plugin loaded `http://127.0.0.1:4185/index.html#per-chat-restore`.
- Browser DOM check confirmed active header icon buttons for Chats and Browser, visible left `Chats panel`, visible right `Browser panel`, and no spec coverage text inside the shell route.
- Browser viewport screenshot confirmed the revised r05 uses r04-style grayscale shell, inline SVG icons, message cards, composer, and plugin panels instead of the rejected annotated shell surface.
- Browser comparison pass against current frontend screenshots confirmed side-panel headers/close buttons were removed; `panelTopbars: 0`, `closeButtons: 0`.
- Browser comparison pass confirmed the current `#per-chat-restore` route uses the app-like composer context strip, chat transcript shape, and minimal Browser panel chrome from the attached frontend screenshots.
- Browser comparison pass confirmed the current `#settings` route includes the Settings rail, search field, `Built by C4OS` filter, active Plugins nav state, and GitHub plugin row from the attached frontend screenshot.
- Static scan confirmed Search is no longer a standalone header plugin icon.
- Static scan confirmed side-panel headers and close buttons remain removed.
- Browser plugin verified `#debug` with header buttons `Chats`, `Files`,
  `Browser`, `Terminal`, `Debug`, and `Settings`; no standalone Search header
  plugin appears.
- Browser plugin verified Chats panel search is present as `Search chats`.
- Browser plugin verified Debug panel shows command terminal output plus
  structured events: `terminal.run`, `tool_call_requested`,
  `tool_output_delta`, and `approval_policy`.
- Browser plugin verified Browser, Terminal, and Debug panel body padding is
  `0px`.
- Browser plugin verified Files panel body padding is `0px` while preserving
  file-tree row spacing.
- The temporary static server remains running for user review at `http://127.0.0.1:4185/`.

## Manual Review Targets

- `./index.html#shell-foundation`
- `./index.html#same-side-replacement`
- `./index.html#per-chat-restore`
- `./index.html#resize-collision`
- `./index.html#hidden-activity`
- `./index.html#debug`
- `./index.html#repair-state`
- `./index.html#settings`
- `./index.html#coverage`
