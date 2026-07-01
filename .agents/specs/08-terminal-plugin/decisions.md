# Terminal Plugin Decisions

Status: proposed

### DEC-001: 023: C4OS Grill Question 023 - Non-File Risk Tool Defaults

Source: `.agents/references/research/final-implementation-import/grill-session/023-c4os-grill-question-023-non-file-risk-tool-defaults.json`

  - What should terminal command execution default to?: Ask by default; remember rules can allow repeated safe commands
  - What should git/worktree, network mutation, and credential use default to?: Allow git/worktree inside trusted projects; ask network and credentials

### DEC-002: 034: C4OS Grill Question 034 - Terminal Session Scope

Source: `.agents/references/research/final-implementation-import/grill-session/034-c4os-grill-question-034-terminal-session-scope.json`

  - What cwd should a terminal use when no project is assigned?: User home directory
  - How many user terminals should one chat session have in v1?: Exactly one user terminal per chat session
  - What happens to the terminal when its chat is removed?: Terminate terminal and delete chat-owned terminal state

### DEC-003: 047: C4OS Grill Question 047 - Terminal Plugin Tool Boundary

Source: `.agents/references/research/final-implementation-import/grill-session/047-c4os-grill-question-047-terminal-plugin-tool-boundary.json`

  - Should the Terminal plugin expose only the user terminal panel or also runtime terminal app tools?: User terminal panel only; runtime terminal tools stay C4OS tool-gateway owned
  - Where should runtime terminal tool activity be visible?: Thread context plus Chat Debug; Terminal panel stays user PTY
  - Which terminal settings belong in config.toml versus Terminal plugin settings?: config.toml owns shell/env defaults and tool policy; plugin settings own panel/UI preferences
  - What environment should the user terminal start with?: User login shell environment plus explicit documented C4OS variables
