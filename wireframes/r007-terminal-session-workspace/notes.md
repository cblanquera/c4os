# Terminal Session Workspace Review Notes

## Round 1 — Shared Expanded Terminal

- Date: 2026-07-15
- Phase: Conceptual wireframes.
- Revision: `r007-terminal-session-workspace`.
- Feedback applied: Restore Expand to Terminal response artifacts, add Stop for a long-running foreground process, and make the expanded artifact operate as one persistent terminal session while the AI composer defaults to Chat.
- Changed behavior:
  - Every Terminal artifact opens the same `shell-1` session in the center workspace.
  - The expanded session shows earlier commands, the selected command, streaming output, and a terminal-local input.
  - A running process accepts stdin and exposes Stop. Stop simulates `Ctrl+C`, marks the command Interrupted with exit 130, and keeps the shell alive.
  - When the shell is idle, terminal-local input creates a new command and matching Terminal response artifact in the conversation.
  - The conversation moves into the contextual left Chat pane and the fixed AI composer remains Chat while Terminal is expanded.
- Simulated or deferred behavior: Output streaming, stdin, process interruption, command execution, and exit values are illustrative in-memory behavior. Real PTY allocation, process signals, filesystem effects, and full-screen interactive programs are deferred.
- Review focus: Inline Stop/Expand hierarchy, shared-session readability, distinction between terminal-local input and the fixed Chat composer, selected-command context, and restore behavior.
- Open questions: None blocking this round.
- Approval path: Approval keeps the Terminal session model in r007; requested refinements create another review round within this revision.

## Round 2 — Continuous Selected Process Body

- Date: 2026-07-15
- Phase: Conceptual wireframes.
- Revision: `r007-terminal-session-workspace`.
- Feedback applied: Remove the expanded command-history list, remove the separate terminal input bar, and make the selected process the main terminal body. After `Ctrl+C`, return `$` directly beneath `^C`.
- Changed behavior:
  - Expanding a Terminal artifact shows only that selected command/process and output as one borderless, scrollable terminal body.
  - Earlier commands remain available as separate response artifacts in the Chat transcript but are not repeated in the expanded workspace.
  - A running process streams into the body and exposes Stop in the workspace header.
  - Stop appends `^C`, marks the process Interrupted with exit 130, and inserts an inline `$` prompt immediately after the output.
  - Submitting the returned prompt creates a new response artifact in `shell-1` and makes that command the new expanded body.
- Simulated or deferred behavior: A process-specific stdin prompt remains conceptually supported when a command requests input, but the running Vite sample does not show an always-present command form. Real PTY behavior remains deferred.
- Review focus: Terminal-body continuity, removal of duplicate history, Stop-to-prompt transition, follow-up command continuity, and distinction from the fixed Chat composer.
- Open questions: None blocking this round.
- Approval path: Approval keeps this selected-process terminal model in r007; requested refinements create another review round within this revision.
