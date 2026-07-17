# Visual QA — r007 Round 1

- Review target: `http://127.0.0.1:4179/index.html#terminal`.
- Inline running artifact:
  - The seeded `npm run dev` response displayed a live Running timer, streaming output, Stop, Expand, Copy, and Reply.
  - Stop appended `^C`, changed the response to Interrupted with exit 130, removed Stop, retained Expand, and preserved `shell-1`.
  - The Terminal composer changed from process-input state back to command-ready state after interruption.
- Expanded Terminal:
  - Expand moved the conversation into the left Chat pane and displayed the complete shared history (`ls` plus `npm run dev`) in the center workspace.
  - The selected command was brought into view and continued receiving simulated stream lines.
  - The terminal-local input accepted stdin while the process was running.
  - After Stop, the same input changed to a shell command prompt; submitting `pwd` with Enter appended a completed Terminal artifact in the conversation and a matching entry in the expanded session.
  - Close restored the centered conversation and the previously selected Terminal composer mode.
- Composer and layout:
  - While expanded, the fixed AI composer computed `data-mode="chat"`, the mode chooser was hidden, and the terminal-local input remained inside the focused artifact.
  - At a 1280 × 720 viewport, the focused Terminal occupied the full 506px artifact stage with zero horizontal document overflow.
- Static checks: JavaScript syntax and whitespace validation passed; all revision files and relative imports were present.

## Round 2 — Continuous Terminal Body

- Review target: `http://127.0.0.1:4179/index.html#terminal`.
- Expanded running process:
  - The focused workspace rendered one `.terminal-main-body` for `npm run dev`; there were zero legacy history-log, history-entry, or separate command-bar containers.
  - The earlier `ls` command remained in session state/transcript history but was not repeated in the expanded workspace.
  - Streaming output continued in the selected process body, Stop remained visible, the fixed AI composer stayed in Chat mode, and no shell prompt appeared while Vite was running.
- Interruption and returned prompt:
  - Stop appended `^C`, changed the selected process to `Interrupted · Exit 130`, hid Stop, and inserted one inline `$` prompt immediately beneath the output.
  - Entering `pwd` at that prompt created a second Terminal response artifact, changed the expanded body to `$ pwd`, showed `/Users/demo/project`, and returned another `$` prompt.
- Layout:
  - The selected process body was borderless and scrollable, with no duplicated card shell around the process output.
  - Browser inspection found zero horizontal document overflow in both running and completed states.
- Static checks: JavaScript syntax and whitespace validation passed after the Round 2 refactor.
