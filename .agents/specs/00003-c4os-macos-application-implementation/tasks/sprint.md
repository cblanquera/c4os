# Spec 00003 Implementation Sprint

Plan state: Accepted 2026-07-18 by the user's advance maintainability-first plan authorization.

Implementation status source: [Spec status](../status.md). Task status source: [Task status](status.md). Normative coverage source: [Coverage ledger](coverage.md).

## Completion Rule

The local macOS development milestone completes only when every task and side quest is `verified`, every task-level Agent Acceptance is `passed`, all 50 Feature Coverage IDs are closed, the integrated acceptance matrix passes, context-promotion review is complete, and the Agent Workspace validator passes.

Task status never uses `accepted`. Every task carries the exact override:

`Acceptance criteria: none — implementation acceptance is delegated to the coordinator’s Agent Acceptance process for Spec 00003.`

## Maintainability-First Sequence

| Order | Task | Output and rationale |
| --- | --- | --- |
| 1 | 00001 Foundation and dependency lock | Create the production Cargo/Tauri/React workspace, lock researched versions, establish typed IPC, test runners, QA isolation, and build commands before feature code. |
| 2 | 00002 Durable core, configuration, and Workspace lifecycle | Establish single-owner persistence, migrations, physical layout, archive safety, recovery, and scoped configuration. |
| 3 | 00003 Policy, credentials, Action Gateway, and execution environments | Put authorization, denial-before-effect, redaction, audit, trusted-root, Git, approval concurrency, and executor boundaries below all facilities. |
| 4 | 00004 Runtime adapters, providers, and capability lifecycle | Implement peer OpenCode/Pi supervision, provider/model discovery, turn binding, preflight, streaming, retry, cancellation, and capability snapshots. |
| 5 | 00005 Native macOS platform and semantic UI foundation | Implement native menu/window/theme/pickers and semantic tokens before broad UI composition. |
| 6 | 00006 Stateful renderer shell and accessible component system | Build the hash-routed Redux projection shell, reusable React Aria primitives, responsive layout, and direct QA routes. |
| 7 | 00007 Workspace, Chat, composer, and conversation | Implement r013 navigation/search, pending chats, transcript/editor, attachments, modes, Reply, branches, focus state, and streaming. |
| 8 | 00008 Artifact framework and File/Folder facilities | Implement shared artifact state/shells, bounded Reply context, brokered File editing, Folder navigation, focus continuity, and stale conflicts. |
| 9 | 00009 Terminal facility | Implement one authorized persistent PTY per Chat, immutable artifacts, streaming, stdin, Stop, recovery, and expanded interaction. |
| 10 | 00010 Native Browser facility | Integrate the proved Rust-owned WKWebView controller, storage profiles, permissions, navigation, focus, clearing, and isolation. |
| 11 | 00011 Plugin and Skill lifecycle | Implement signed immutable packages, marketplaces, declarative plugins, progressive skills, hooks, revocation, update, rollback, and Settings state. |
| 12 | 00012 MCP lifecycle | Implement 2025-11-25 STDIO/Streamable HTTP definitions, trust, supervision, policy mediation, bounded execution, restart, revocation, and failure UX. |
| 13 | 00013 Onboarding and Settings integration | Complete Providers, Models, Runtimes, Plugins, Skills, MCP, Configuration, and Advanced Policies against real services. |
| 14 | 00014 Updates, recovery, diagnostics, and degraded-state integration | Close independent update channels, last-known-good behavior, restart recovery, redacted diagnostics, and honest failure/degraded surfaces. |
| 15 | 00015 Deterministic QA and integrated Agent Acceptance | Run the complete production-rendered r013 matrix and all Rust, adapter, security, persistence, native, accessibility, responsive, failure, and recovery suites. |

## Required Side Quests

- `00015A`: independent security and denial-before-side-effect audit.
- `00015B`: accessibility, native macOS, keyboard, responsive, theme, focus, and Computer Use audit.
- `00015C`: 50-ID coverage, evidence, documentation, context-promotion, validator, worktree, and commit-history audit.

These side quests are required cross-checks, not substitutes for parent-task verification.

## Parallel Work Rules

- The coordinator owns `tasks/`, root manifests, shared schemas, cross-domain state contracts, integration, and commits.
- Concurrent writers receive explicit non-overlapping module ownership. Shared contracts are changed by the coordinator or through a serialized handoff.
- Every subagent result is diff-inspected and rerun by the coordinator before integration.
- Proof code stays isolated; production code may reuse learned constraints but not proof implementations as verification.

## Verification Loop

For every task: mark `started`, implement, run focused tests, launch affected surfaces, record evidence, run Agent Acceptance, fix failures, run broader regressions, close mapped coverage only after supporting tasks also pass, mark `verified`, and make a coherent local checkpoint commit.

## External Gates

Signing, notarization, distributable packaging, signed-updater evidence, Windows/Linux claims, public marketplace governance, Codex import, active cross-runtime migration, full-screen/password-entry terminal behavior, browser sub-tabs, multiple Reply targets, detached Chat windows, and unvalidated named SSH profiles remain explicit gates from the Frozen contract. They do not weaken the local macOS development build requirements.
