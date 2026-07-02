# Terminal Plugin Decisions

Status: proposed

### DEC-001: 023: C4OS Grill Question 023 - Non-File Risk Tool Defaults

Source: `.agents/references/research/final-implementation-import/grill-session/023-c4os-grill-question-023-non-file-risk-tool-defaults.json`

  - What should terminal command execution default to?: Ask by default; remember rules can allow repeated safe commands
  - What should git/worktree, network mutation, and credential use default to?: Allow git/worktree inside trusted projects; ask network and credentials

### DEC-002: 034: C4OS Grill Question 034 - Terminal Session Scope

Source: `.agents/references/research/final-implementation-import/grill-session/034-c4os-grill-question-034-terminal-session-scope.json`

  - What cwd should a terminal use when no project is assigned?: User home directory
  - How many user terminals should one chat session have?: Exactly one user terminal per chat session
  - What happens to the terminal when its chat is removed?: Terminate terminal and delete chat-owned terminal state

### DEC-003: 047: C4OS Grill Question 047 - Terminal Plugin Tool Boundary

Source: `.agents/references/research/final-implementation-import/grill-session/047-c4os-grill-question-047-terminal-plugin-tool-boundary.json`

  - Should the Terminal plugin expose only the user terminal panel or also runtime terminal app tools?: User terminal panel only; runtime terminal tools stay C4OS tool-gateway owned
  - Where should runtime terminal tool activity be visible?: Thread context plus Chat Debug; Terminal panel stays user PTY
  - Which terminal settings belong in config.toml versus Terminal plugin settings?: config.toml owns shell/env defaults and tool policy; plugin settings own panel/UI preferences
  - What environment should the user terminal start with?: User login shell environment plus explicit documented C4OS variables

### DEC-004: Backend App Architect Resolution - Terminal

Source: 2026-07-02 user architect profile in active chat.

  - Boundary: The Terminal plugin owns the user PTY panel only. Runtime
    `terminal.run` and similar agent tools remain C4OS tool-gateway calls with
    structured events, approval policy, audit, and Chat Debug/thread context
    visibility.
  - Platform adapter: PTY behavior uses a terminal backend abstraction.
    macOS/Linux use PTY-compatible implementations; Windows uses ConPTY or a
    documented Windows terminal backend fallback. Shell/env defaults are
    platform-specific and stored in config policy.
  - Resource control: User PTY sessions and runtime terminal tools define
    output buffering, backpressure, scrollback limits, idle handling, process
    termination, and memory caps before implementation.
  - Event model: PTY create, input, output chunk, resize, cwd change, env
    snapshot, exit, terminate, and chat-delete cleanup emit typed events.
  - Maintainability: Terminal backend, renderer adapter, session lifecycle,
    policy, environment building, output transport, and settings UI are
    separate modules with comments on platform-specific behavior.

### DEC-005: POC Result - User PTY Lifecycle And Runtime Separation

Source: `proofs/terminal-user-pty-lifecycle/`

  - The Terminal plugin owns exactly one user PTY per chat session.
  - Assigned project chats start the user PTY in the project cwd; unassigned
    chats start in the user home directory.
  - Runtime `terminal.run` and similar agent tools remain tool-gateway events
    visible in thread context and Chat Debug, not Terminal panel state.
  - Removing a chat terminates the user PTY and deletes chat-owned terminal
    state.
