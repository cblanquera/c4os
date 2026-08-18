# Spec 00003 Implementation Sprint

Plan state: Original phase accepted 2026-07-18. Corrective r013 convergence phase accepted 2026-07-27 by the user's explicit request for new tasks, a less restrictive practical journey, and production shape/functionality as close to r013 as possible.

Implementation status source: [Spec status](../status.md). Task status source: [Task status](status.md). Normative coverage source: [Coverage ledger](coverage.md).

## Completion Rules

Tasks 00001 through 00015C preserve the historical local-development verification record and its coordinator-owned Agent Acceptance model. The 2026-07-27 human audit supersedes that record only for current visible-product readiness; it does not erase the underlying security, service, persistence, or test evidence.

Corrective Tasks 00016 through 00022 and their required review side quests complete only when verification passes and the user explicitly accepts each named human-reviewable result. Their allowed statuses are `open`, `started`, `verified`, and `accepted`. Task 00022 cannot complete at `verified`; unresolved review findings become new open tasks.

The current corrective milestone completes only when Tasks 00016 through 00022 and required review side quests are `accepted`, the corrective coverage supplement is closed, the integrated r013 comparison passes, context-promotion review is complete, and the Agent Workspace validator passes.

## Corrective Journey Contract

1. A new user enters provider details in the compact r013 onboarding form.
2. Test Connection uses the submitted credential transiently and does not persist it or ask for duplicate C4OS approval.
3. A successful current test keeps the compact form and automatically selects the viable model with the most normalized C4OS-supported features; onboarding shows no model picker or default-confirmation step.
4. Continue persists the provider and secure credential reference once, then enters Workspace Start.
5. If macOS credential storage genuinely fails, the user sees one exceptional recovery choice with Retry; no UI implies that a C4OS prompt is an Apple permission dialog.
6. Exact direct user controls count as intent for their named action and target. Runtime-, agent-, extension-, changed-target-, destructive-, explicit-`Ask`-, and managed-policy actions retain the appropriate approval or denial path.
7. The returning user reaches the compact r013 Workspace, Settings, Chat, composer, model, and artifact journeys without implementation commentary or responsibility duplicated across routes.

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

## Corrective r013 Convergence Sequence

| Order | Task | Output and rationale |
| --- | --- | --- |
| 16 | [00016 Practical direct intent and credentials](00016-practical-direct-intent-and-credentials.md) | Repair the normal Keychain path and remove redundant direct-action prompts before reshaping user journeys. |
| 17 | [00017 r013 visual foundation and shell geometry](00017-r013-visual-foundation-and-shell-geometry.md) | Establish shared r013-derived geometry and density so route work does not accumulate independent CSS patches. |
| 18 | [00018 r013 onboarding and Workspace Start](00018-r013-onboarding-and-workspace-start.md) | Compose the first-run and returning-user launch journeys on Tasks 00016 and 00017. |
| 19 | [00019 r013 Settings information architecture](00019-r013-settings-information-architecture.md) | Restore compact Settings hierarchy and provider/model responsibility boundaries against real services. |
| 20 | [00020 r013 Workspace, Chat, and composer](00020-r013-workspace-chat-and-composer.md) | Converge the primary work surface, model navigation, capability feedback, transcript, and composer. |
| 21 | [00021 r013 artifact and responsive convergence](00021-r013-artifact-and-responsive-convergence.md) | Align every artifact shell and focus/contextual state after the shared Workspace composition stabilizes. |
| 22 | [00022 r013 integrated human acceptance](00022-r013-integrated-human-acceptance.md) | Run the complete production-rendered comparison and require explicit user acceptance before closeout. |
| 22A | [00022A onboarding automatic model selection](00022A-onboarding-automatic-model-selection.md) | Resolve the user's review finding by removing the post-Test picker and enforcing strongest-supported automatic selection. |
| 22B | [00022B onboarding Continue recovery](00022B-onboarding-continue-recovery.md) | Resolve the user's blocking production finding by migrating the legacy app configuration placeholder and preserving secure retry after a failed Continue. |

## Required Side Quests

- `00015A`: independent security and denial-before-side-effect audit.
- `00015B`: accessibility, native macOS, keyboard, responsive, theme, focus, and Computer Use audit.
- `00015C`: 50-ID coverage, evidence, documentation, context-promotion, validator, worktree, and commit-history audit.
- `00022A`: onboarding automatic-model correction discovered during Task 00022 user review.
- `00022B`: onboarding Continue/configuration recovery discovered during Task 00022 user review.

These side quests are required cross-checks, not substitutes for parent-task verification.

## Parallel Work Rules

- The coordinator owns `tasks/`, root manifests, shared schemas, cross-domain state contracts, integration, and commits.
- Concurrent writers receive explicit non-overlapping module ownership. Shared contracts are changed by the coordinator or through a serialized handoff.
- Every subagent result is diff-inspected and rerun by the coordinator before integration.
- Proof code stays isolated; production code may reuse learned constraints but not proof implementations as verification.

## Verification Loop

For Tasks 00001 through 00015C, preserve their recorded verification loop. For each corrective task: mark `started`, implement, run focused tests, launch affected surfaces, compare them with r013 at matched states and viewports, record evidence, fix failures, run broader regressions, mark `verified`, and present the named visual/functional artifact to the user. Mark `accepted` only after explicit user review. A local checkpoint commit still requires separate user authorization.

## External Gates

Signing, notarization, distributable packaging, signed-updater evidence, Windows/Linux claims, public marketplace governance, Codex import, active cross-runtime migration, full-screen/password-entry terminal behavior, browser sub-tabs, multiple Reply targets, detached Chat windows, and unvalidated named SSH profiles remain explicit gates from the Frozen contract. They do not weaken the local macOS development build requirements.
