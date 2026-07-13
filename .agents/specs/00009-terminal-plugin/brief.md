# Terminal Plugin Brief

Status: Proposed

## Goal

Define one platform-backed user PTY per chat, cwd selection, lifecycle cleanup, output transport, Terminal UI preferences, Windows ConPTY provisions, and strict separation from runtime terminal tool calls.

## Boundaries

- Planning only; no implementation is authorized.
- The user PTY must not become the runtime-owned command terminal.
- Shared truth comes from Context Files, not sibling specs.

## Sources

- [C4OS Context router](../../context/index.md)
- [Legacy Terminal package](../../resources/history/specs/08-terminal-plugin/index.md)
