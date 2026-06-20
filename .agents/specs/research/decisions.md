# Research Decisions

## DEC-001: Greenfield Restart

Status: accepted
Confidence: imported
MVP: yes
Phase: mvp
Source: `plans/product-brief.md`

C4OS is being treated as a greenfield restart from current plan documents. Existing prior planning records are product intent and evidence, not a required implementation sequence.

## DEC-002: Folder-First Trust Gate

Status: accepted
Confidence: imported
MVP: yes
Phase: mvp
Source: `plans/product-brief.md`

The first-run and main workspace experience is folder-first. Prompting is disabled until a trusted folder is available.

## DEC-003: Product Model Is App-Owned

Status: accepted
Confidence: imported
MVP: yes
Phase: mvp
Source: `plans/product-brief.md`

The product model centers app-owned workspaces, sessions, runs, approvals, artifacts, providers, files, browser, terminal, and settings rather than exposing runtime internals as the primary UX.

## DEC-004: Browser Promotion Requires Isolation Proof

Status: accepted
Confidence: superseded
MVP: yes
Phase: mvp
Source: `plans/product-brief.md`

Native Browser implementation was initially expected to wait until isolation proof. This is superseded by grill decisions that promote Browser to MVP product scope while preserving agent authority and audit boundaries.

## DEC-005: Extension Runtime Impact Requires Explicit Enablement

Status: accepted
Confidence: user-accepted
MVP: yes
Phase: mvp
Source: `plans/product-brief.md`, `.agents/references/research/grill-session.md`

MVP includes install/connect flows for plugins, skills, and MCP servers. Extensions can affect runtime execution, model context, tools, hooks, or app-owned state only after explicit per-extension enablement. Skills and plugins may be invoked implicitly through "use when" routing or explicitly by the user. MCP servers require explicit user invocation.

## DEC-006: Thin Runtime Adapter For MVP

Status: accepted
Confidence: user-accepted
MVP: yes
Phase: mvp
Source:
- `.agents/specs/research/poc/results.md`
- `proofs/opencode-runtime/opencode-runtime-evidence-2026-06-20.md`
- `proofs/pi-runtime/pi-runtime-evidence-2026-06-20.md`
- `.agents/references/research/grill-session.md`
Related:
- Q-001
- ASM-001
- RISK-001
- TASK-002

C4OS should freeze the MVP around OpenCode first behind a thin app-owned runtime adapter rather than exposing OpenCode as the product model. OpenCode is the first backend because it supports connectors and has stronger server-backed session/event ergonomics for the first implementation. Pi remains a later adapter target. Product records, UI, approvals, artifacts, provider settings, extension enablement, and persistence stay C4OS-owned.

## DEC-007: Browser MVP Is User-Owned Project Browser

Status: accepted
Confidence: user-accepted
MVP: yes
Phase: mvp
Source:
- `.agents/specs/research/poc/results.md`
- `proofs/native-browser-plugin/native-browser-plugin-evidence-2026-06-20.md`
- `proofs/native-browser-wry/native-browser-wry-evidence-2026-06-20.md`
- `proofs/native-browser-tauri/native-browser-tauri-evidence-2026-06-20.md`
- `.agents/references/research/grill-session.md`
Related:
- Q-002
- RISK-002
- TASK-003

The MVP Browser tab is promoted as a user-owned desktop Browser with project-scoped profile, public web access, local file browsing, request-scoped agent browsing, and logged-in Browser use when the user's request clearly requires it. Browser downloads are not MVP. The earlier POCs remain implementation risk evidence, especially around direct Tauri WebviewWindow internals, but they do not remove Browser from MVP scope.

## DEC-008: Terminal MVP Requires Backend-Owned Lifecycle

Status: accepted
Confidence: user-accepted
MVP: yes
Phase: mvp
Source:
- `.agents/specs/research/poc/results.md`
- `proofs/native-terminal-plugin/native-terminal-plugin-evidence-2026-06-20.md`
- `proofs/native-terminal-tauri/native-terminal-tauri-evidence-2026-06-20.md`
- `.agents/references/research/grill-session.md`
Related:
- Q-003
- RISK-002
- TASK-004

The MVP Terminal tab is promoted as two backend-owned surfaces: a user terminal and an agent-owned command terminal. Renderer shell spawn is rejected. The agent-owned command terminal may run read-only allowlisted commands without prompting, but non-allowlisted commands inside the active project require per-command approval and outside-project commands require explicit permission. The implementation must preserve trusted-root cwd validation, sanitized environment, bounded renderer transport, audit records, cancellation, failure reporting, and cleanup.

## DEC-010: Deterministic Command Allowlist

Status: accepted
Confidence: user-accepted
MVP: yes
Phase: mvp
Source: `.agents/references/research/grill-session.md`
Related:
- REQ-015
- Q-003

Agent-owned terminal read-only command execution is governed by deterministic exact pattern matching against an app-bundled default allowlist, with workspace config able to override or extend the defaults. The MVP default allowlist includes local read commands and network read commands such as `curl`.

## DEC-009: r04 Wireframes Are Behavioral Handoff

Status: accepted
Confidence: evidence-backed
MVP: yes
Phase: mvp
Source:
- `wireframes/screens.md`
- `wireframes/ui-handoff-spec.md`
- `wireframes/r04-single-page-app/README.md`
- `wireframes/r04-single-page-app/qa/notes.md`
Related:
- TASK-005
- EVD-009
- AC-009
- AC-010
- AC-011
- AC-012
- AC-013
- AC-014
- AC-015

r04 is accepted as the implementation behavior handoff for MVP UI scope, not as final brand styling or hardcoded sample data. Implementation should preserve the desktop shell, state model, composer controls, structured thread events, right-tool boundaries, and settings IA while sourcing real project, provider, model, file, runtime, plugin, skill, MCP, terminal, and copy values from product state or later copy decisions.

## DEC-011: Chat Prompt Extension Tags

Status: accepted
Confidence: user-accepted
MVP: yes
Phase: mvp
Source: `.agents/references/research/grill-session.md`
Related:
- CAP-009
- CON-004

C4OS chat prompt syntax uses `$` to explicitly invoke skills, `@` to explicitly invoke plugins, and `^` to explicitly invoke MCP servers. Skills and plugins may also be selected implicitly by enabled "use when" routing. MCP servers are explicit-invocation only.
