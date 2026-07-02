# Chat Debug Plugin Decisions

Status: proposed

### DEC-001: 035: C4OS Grill Question 035 - Chat Debug Visibility

Source: `.agents/references/research/final-implementation-import/grill-session/035-c4os-grill-question-035-chat-debug-visibility.json`

  - Should Chat Debug be enabled by default?: Disabled by default; developer-oriented plugin
  - What should Chat Debug show?: Other
  - Notes: cli commands and results, tool use and tool events

### DEC-002: 043: C4OS Grill Question 043 - Chat Debug Redaction And History

Source: `.agents/references/research/final-implementation-import/grill-session/043-c4os-grill-question-043-chat-debug-redaction-and-history.json`

  - Should Chat Debug ever be enabled by default?: Disabled by default everywhere
  - Which Chat Debug values must always be redacted?: Secrets, tokens, raw credentials, provider keys, auth headers, cookies, and sensitive params
  - Should Chat Debug show historical runs?: Current and historical runs for the active chat session
  - Should users be able to export Chat Debug logs?: No export
  - Where should approval decisions appear?: Both thread context and Chat Debug
  - Sensitive plugin setting boundary: Plugin settings marked `sensitive` are
    treated as secret-like values. Chat Debug must never display or persist raw
    sensitive plugin setting values.
  - Refinement source: 2026-07-02 grill intake, "Plugin Sensitive Settings
    Storage".

### DEC-003: 047: C4OS Grill Question 047 - Terminal Plugin Tool Boundary

Source: `.agents/references/research/final-implementation-import/grill-session/047-c4os-grill-question-047-terminal-plugin-tool-boundary.json`

  - Should the Terminal plugin expose only the user terminal panel or also runtime terminal app tools?: User terminal panel only; runtime terminal tools stay C4OS tool-gateway owned
  - Where should runtime terminal tool activity be visible?: Thread context plus Chat Debug; Terminal panel stays user PTY
  - Which terminal settings belong in config.toml versus Terminal plugin settings?: config.toml owns shell/env defaults and tool policy; plugin settings own panel/UI preferences
  - What environment should the user terminal start with?: User login shell environment plus explicit documented C4OS variables
