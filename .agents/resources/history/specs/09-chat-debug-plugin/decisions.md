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

### DEC-004: Backend App Architect Resolution - Chat Debug

Source: 2026-07-02 user architect profile in active chat.

  - Observability role: Chat Debug is the developer/operator observability
    plugin for typed C4OS events. It displays tool calls, runtime events,
    approvals, plugin lifecycle events, terminal tool events, attachment
    adaptation, structured errors, and redacted audit summaries for the active
    chat.
  - Redaction floor: Redaction happens before display and before debug
    persistence. Secrets, tokens, credentials, auth headers, cookies,
    sensitive params, raw provider keys, and plugin settings marked
    `sensitive` are never stored in raw debug records.
  - History bounds: Historical run display is per active chat session, bounded
    by configurable count/size limits, and prunable with chat deletion. No
    export exists in accepted scope.
  - Worker-friendly UX: Chat Debug remains disabled by default and advanced,
    but labels should explain activity in plain language for support,
    operations, and admin users who need to diagnose app behavior.
  - Maintainability: Debug event schema, redaction, storage, filtering,
    rendering, and retention are separate modules with testable fixtures.

### DEC-005: POC Result - Chat Debug Redaction And History

Source: `proofs/chat-debug-redaction-history/`

  - Result: Passed with `node --test
    proofs/chat-debug-redaction-history/proof.test.mjs`.
  - Decision: Chat Debug can use typed per-chat debug events with chronological
    inspection, newest-run display, bounded retention, approval visibility in
    both thread context and Chat Debug, redaction before persistence/display,
    chat deletion cleanup compatibility, and no export surface.
  - Reconciliation: This preserves prior approval visibility decisions from
    `proofs/approval-ui-flow/`, prior structured tool event decisions from
    runtime/tool proofs, and the sensitive plugin setting boundary from this
    spec.
