# Terminal Plugin Acceptance

Status: proposed

| ID | Acceptance Criteria |
| --- | --- |
| AC-001 | User PTY and runtime terminal tool activity are visibly separate. |
| AC-002 | Terminal starts with user login shell environment plus documented C4OS variables. |
| AC-003 | Windows ConPTY or explicit Windows fallback behavior is documented before freeze. |
| AC-004 | PTY output transport includes bounded buffering, backpressure, scrollback limit, and cleanup behavior. |
| AC-005 | Terminal events distinguish user input/output from runtime tool output without sharing the Terminal panel as source of truth for agent commands. |
