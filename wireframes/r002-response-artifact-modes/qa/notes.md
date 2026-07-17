# Browser QA Notes

## Round 1 - 2026-07-15

- Entry point: `http://127.0.0.1:4174/workflows.html`.
- Review screen: `http://127.0.0.1:4174/index.html`.
- Captures: `round-1-desktop.png`, `round-1-artifact-pane.png`, and `round-1-narrow.png`.
- Browser mode activation updated the selected tab, visible panel, URL hash, and persistent composer state.
- Direct Chat submission appended a user/assistant pair; direct Files opened `README.md`; direct Terminal ran `pwd`; each selected mode remained active after submission.
- Opening a new URL appended one Browser artifact and left the right pane closed.
- Replying to `browser-1` changed its URL to `https://example.com/dashboard`, updated its status and navigation history, and left the Browser artifact count unchanged.
- Replying to `file-1` exposed an approval-required state. Approving saved the proposed contents immediately and advanced Version 1 to Version 2.
- Replying to `terminal-1` left its `$ ls` card unchanged and appended `$ du -sh .` with `42M` output in `shell-1`.
- Expanding Browser, File, and Terminal produced three reusable right-pane tabs. Closing the Terminal tab preserved both Terminal transcript artifacts.
- The expanded File viewer began read only, entered Edit mode explicitly, and saved revised contents back to the inline card as Version 2.
- Contextual Browser Copy placed the current artifact URL on the clipboard and displayed the Copy notifier.
- At 720 by 900, both panels closed into overlays and the document width remained 720px without horizontal overflow.
- Browser console check returned no warnings or errors.

## Round 2 - 2026-07-15

- Captures: `round-2-message-hierarchy.png` and `round-2-artifact-actions.png`.
- The existing assistant response renders `Worked for 18 sec` before `GPT-5`; a newly submitted Chat response uses the same DOM order.
- The Browser artifact icon center, identity-block center, and header center measured at the same vertical coordinate.
- Browser, File, and Terminal headers contain only their Expand control on the right.
- Browser, File, and Terminal Copy/Reply actions appear after the artifact footer and reveal on hover or keyboard focus.
- Browser Copy remained functional and copied the current URL from its relocated control.
- A newly created Browser artifact used the revised header and lower-left action structure.
- At 720 by 900, both panels remained overlays and the document retained zero horizontal overflow.
- Browser console check returned no warnings or errors.

## Round 3 - 2026-07-15

- Capture: `round-3-response-hierarchy.png`.
- The ordinary assistant response DOM order is thinking disclosure, assistant icon, and response content containing the model label and bubble.
- Browser, File, and Terminal artifacts each contain a direct thinking disclosure, assistant icon/model row, bordered artifact surface, and lower action row.
- A newly submitted Chat response used the unified assistant hierarchy.
- A newly generated `README.md` File artifact included `Worked for 11 sec`, the active `GPT-5` label, and its bordered artifact surface.
- The Browser artifact type-icon box and SVG centers matched exactly on both axes with a zero-pixel delta.
- At 720 by 900, the document retained zero horizontal overflow and all three initial artifact preambles remained present.
- Browser console check returned no warnings or errors.

## Round 4 - 2026-07-15

- Capture: `round-4-aligned-widths.png`.
- The initial assistant thinking disclosure, icon/model row, and bubble each begin at the same 256px left coordinate.
- A newly submitted Chat response used the same child order and matching left coordinates.
- Initial and newly generated Terminal artifact surfaces measured approximately 81% of the available desktop thread width instead of spanning the thread.
- Artifact surfaces remain left aligned with assistant responses.
- At 720 by 900, the document retained zero horizontal overflow.
- Browser console check returned no warnings or errors.

## Round 5 - 2026-07-15

- Capture: `round-5-shared-response-containers.png`.
- Initial assistant and Browser artifact containers both computed to `16px 16px 16px 5px` with a 5px lower-left radius and 1px border.
- Four initial response surfaces use the shared `.response-container` class: one assistant bubble and three artifact surfaces.
- Newly generated Chat and Browser responses inherited the same shared radius and container class.
- At 720 by 900, checked response surfaces retained a 5px lower-left radius and the document retained zero horizontal overflow.
- Browser console check returned no warnings or errors.
