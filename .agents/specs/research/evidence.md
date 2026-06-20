# Research Evidence

## EVD-001: Product Brief

Status: accepted
Confidence: imported
MVP: yes
Phase: mvp
Source: `plans/product-brief.md`

The product brief defines the greenfield restart, product thesis, target users, user problems, principles, feature areas, data model concepts, security requirements, success metrics, and risks.

## EVD-002: Interface Brief

Status: accepted
Confidence: imported
MVP: yes
Phase: mvp
Source: `plans/product-interface.md`

The interface brief defines the three-panel shell, prompt composer, model selector behavior, messenger-style session thread, Browser/Files/Terminal tab rules, and settings screens.

## EVD-003: Wireframe Pegs

Status: accepted
Confidence: imported
MVP: yes
Phase: mvp
Source: `plans/pegs/*.png`

The PNG pegs provide visual references for app start, new session, chat session, provider/model popovers, settings screens, file explorer/editor, and terminal.

## EVD-004: Runtime Adapter POC Plan

Status: proposed
Confidence: proposed
MVP: yes
Phase: poc
Source: `.agents/specs/research/poc/brief.md`
Related:
- Q-001
- ASM-001
- RISK-001
- TASK-002

The planned runtime adapter proof compares OpenCode Runtime and Pi Runtime through the same minimal adapter contract, with runnable implementations expected under `proofs/opencode-runtime/` and `proofs/pi-runtime/`.

## EVD-005: OpenCode Runtime POC

Status: accepted
Confidence: evidence-backed
MVP: yes
Phase: poc
Source: `proofs/opencode-runtime/opencode-runtime-evidence-2026-06-20.md`
Related:
- Q-001
- ASM-001
- RISK-001
- TASK-002
- DEC-006

OpenCode 1.17.8 and `@opencode-ai/sdk` 1.17.8 proved local server startup, project/path inspection, session creation and resume, SDK event streaming, permission configuration, and session abort. Prompt execution and live permission-request capture remain partial because the proof intentionally avoided provider credentials and token spend.

## EVD-006: Pi Runtime POC

Status: accepted
Confidence: evidence-backed
MVP: yes
Phase: poc
Source: `proofs/pi-runtime/pi-runtime-evidence-2026-06-20.md`
Related:
- Q-001
- ASM-001
- RISK-001
- TASK-002
- DEC-006

Pi `@earendil-works/pi-agent-core` 0.79.8 and `@earendil-works/pi-coding-agent` 0.79.8 proved session creation and resume, prompt execution through the Agent loop, lifecycle/tool event streaming, pre-execution tool interception, approval denial before tool execution, and run lifecycle control. Filesystem/process/network/credential sandboxing remains app-owned.

## EVD-007: Browser Isolation POCs

Status: accepted
Confidence: evidence-backed
MVP: yes
Phase: poc
Source:
- `proofs/native-browser-plugin/native-browser-plugin-evidence-2026-06-20.md`
- `proofs/native-browser-wry/native-browser-wry-evidence-2026-06-20.md`
- `proofs/native-browser-tauri/native-browser-tauri-evidence-2026-06-20.md`
Related:
- Q-002
- DEC-004
- DEC-007
- RISK-002
- TASK-003

Browser isolation is viable for a constrained preview/browser surface. The sandboxed browser proof passed, raw Wry passed with a `window.ipc` warning and no registered privileged handler, and Tauri WebviewWindow failed the fully unbridged surface requirement because page content observed `window.__TAURI_INTERNALS__`. Grill review later promoted Browser to MVP product scope, so this evidence is now an implementation-risk record rather than a deferral decision.

## EVD-008: Terminal Lifecycle POCs

Status: accepted
Confidence: evidence-backed
MVP: yes
Phase: poc
Source:
- `proofs/native-terminal-plugin/native-terminal-plugin-evidence-2026-06-20.md`
- `proofs/native-terminal-tauri/native-terminal-tauri-evidence-2026-06-20.md`
Related:
- Q-003
- DEC-008
- RISK-002
- TASK-004

Backend-owned terminal lifecycle is viable on macOS. Python and Rust PTY proofs passed trusted-root validation, sanitized process ownership, input/output streaming, resize observation, approval-gated launch decisions, failure reporting, cancellation, and cleanup. Grill review promoted Terminal to MVP product scope with separate user and agent-owned command terminal surfaces, deterministic allowlist, approvals, and audit records.

## EVD-009: r04 Wireframe Handoff

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
- REQ-009
- REQ-010
- REQ-011
- REQ-012
- REQ-013
- REQ-014
- AC-009
- AC-010
- AC-011
- AC-012
- AC-013
- AC-014
- AC-015
- DEC-009
- TASK-005

The r04 single-page wireframe handoff provides the current implementation baseline for app start, new session, chat session, provider/model selectors, Browser, Files, Terminal, and Settings surfaces. The promoted evidence is behavioral: layout regions, state ownership, navigation relationships, trust boundaries, composer controls, structured event surfaces, tool-tab boundaries, and settings IA. Placeholder content and visual styling remain illustrative.
