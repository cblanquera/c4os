# Browser QA Notes

## Round 1 - 2026-07-15

- Review screen: `http://127.0.0.1:4175/index.html#chat`.
- Captures: `round-1-chat-mode.png` and `round-1-mode-popover.png`.
- The compact Chat trigger renders directly before the paperclip attachment control.
- Opening the trigger exposes a four-option `menu` with Chat checked and focus moved to the selected option.
- Selecting Files, Browser, or Terminal updates the composer `data-mode`, trigger label/icon, matching input panel, and URL hash; the popover closes after selection.
- Files mode retains the mode trigger while hiding Chat-only attachment and microphone controls.
- Starting a Browser artifact reply hides the mode trigger and shows the natural-language reply strip; cancelling restores the prior Files mode and trigger.
- Escape closes the open popover and returns keyboard focus to its trigger.
- At the reviewed 1280 by 720 desktop viewport, document width equals viewport width with no horizontal overflow.
- Static responsive rules constrain the popover to the available viewport width below 620px.

## Round 2 - 2026-07-15

- Captures: `round-2-browser-header.png`, `round-2-file-editor.png`, and `round-2-terminal-command.png`.
- Browser artifacts render a single toolbar header ordered as browser icon, Back, Forward, Refresh, read-only current address, and Expand.
- A Browser reply updated the header address to `https://example.com/dashboard`; the address remained read only and Back became enabled.
- File read mode exposes Edit and Expand. Edit mode replaces Edit with Cancel and Save, shows line numbers, and uses an auto-scrolling, vertically resizable textarea.
- Adding a fifth editor line generated line number 5 automatically.
- File Cancel restored the exact original 226.19px artifact height; Save updated the preview, status, and illustrative version before returning to read mode.
- Initial Terminal output omits `$ ls` while its header title is `$ ls`.
- A generated `pwd` artifact used `$ pwd` as its header and `/Users/demo/project` as its output-only body.
- Files, Browser, and Terminal leading composer controls each measured 30px by 30px; Files Browse has no visible text label.
- Newly generated Browser and File artifacts inherited the consolidated toolbar and inline editor controls.

## Round 3 - 2026-07-15

- Captures: `round-3-browser-icon.png`, `round-3-file-icon.png`, and `round-3-terminal-icon.png`.
- Browser, File, and Terminal artifacts each render their type-specific SVG in the model identity row beside `GPT-5`.
- None of the three initial artifact headers contain an `.artifact__icon` after the change.
- Newly generated Browser, File, and Terminal artifacts also contain a model-row icon and no header icon.
- The Browser toolbar now begins with Back, followed by Forward, Refresh, read-only address, and Expand.
- File and Terminal headers begin directly with their title/metadata block.
- The Terminal composer’s 30px prefix box ends 8px before the command field begins, with no geometric overlap.

## Round 4 - 2026-07-15

- Capture: `round-4-browser-pane-header.png`.
- Expanding the inline Browser artifact opens the right pane with an active `Example Domain` tab.
- The expanded Browser viewer renders a dedicated `.pane-browser-bar` with Back, Forward, Refresh, and an editable address field in one row.
- The toolbar measured 379px by 47px; each navigation button measured 30px by 30px.
- The toolbar computed to grid layout and did not wrap or overlap at the reviewed pane width.
- Submitting `https://example.com/dashboard` from the pane address updated the shared Browser state.
- Existing user-authored style adjustments outside the pane Browser selectors were left unchanged.
