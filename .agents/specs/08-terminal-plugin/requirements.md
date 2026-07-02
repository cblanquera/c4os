# Terminal Plugin Requirements

Status: proposed

| ID | Requirement |
| --- | --- |
| REQ-001 | Terminal plugin exposes only the user terminal panel. |
| REQ-002 | Exactly one user terminal per chat session. |
| REQ-003 | No assigned project terminal cwd is user home directory. |
| REQ-004 | Removing a chat terminates the terminal and deletes chat-owned terminal state. |
| REQ-005 | Runtime terminal tools remain C4OS tool-gateway owned and visible in thread context plus Chat Debug. |
| REQ-006 | config.toml owns shell/env defaults and tool policy; plugin settings own panel/UI preferences. |
| REQ-007 | Define terminal backend abstraction with Windows ConPTY or fallback provision, plus macOS/Linux PTY behavior. |
| REQ-008 | Define output buffering, backpressure, scrollback limits, idle behavior, process cleanup, and memory caps. |
| REQ-009 | Emit typed terminal lifecycle/input/output/resize/exit events for user PTY state. |
| REQ-010 | Keep runtime terminal tool events structurally separate from user PTY panel events. |
