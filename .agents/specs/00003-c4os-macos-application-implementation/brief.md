# Brief

## User Goal

Leave Specs `00001-c4os-ai-harness-research` and `00002-platform-visual-theme-feasibility` as research-only packages and put the actual implementation details in a new spec.

## Outcome

Produce the production implementation contract for a macOS-first C4OS desktop application. The contract combines accepted runtime/security behavior, the accepted r013 product experience, and the accepted macOS visual/theme boundary without converting research evidence or wireframes into implementation claims.

## In Scope

- A Tauri 2 application with a Rust-authoritative C4OS core and an unprivileged web renderer.
- The complete accepted r013 product surface as the forward UI contract.
- Workspace, project, session, turn, run, artifact, approval, credential-reference, configuration, provenance, and audit state.
- Peer OpenCode and Pi runtime adapters behind one C4OS-owned normalized contract.
- Local execution plus the accepted Docker and Remote SSH abstraction, with named targets gated by target-specific validation.
- Capability-aware models, attachments, session controls, streaming, Retry, cancellation, and recovery.
- Approval policy, trusted-root enforcement, single-use authorizations, and the privileged tool gateway.
- Files, Browser, and Terminal Response Artifacts through brokered native boundaries.
- Providers, models, runtimes, plugins, skills, MCP servers, configuration, and Advanced Policies.
- Complete supported Plugin, Skill, and MCP installation, activation, execution, supervision, revocation, update, rollback, and failure behavior; metadata-only Settings facades are insufficient.
- macOS-native menus, standard window decorations, semantic system-following light/dark presentation, accessibility, and production QA.
- Separate application, runtime, and plugin update channels with rollback.

## Initial Target Boundary

- Product target: macOS desktop.
- Evidenced feasibility target: macOS 26.5.1 arm64.
- First implementation and review milestone: local macOS development build.
- Windows and Linux production claims, packaging, native appearance, process enforcement, and visual acceptance are excluded until separately researched and proved. The selected macOS native-WebKit Browser boundary passed its required pre-Freeze Proof; production availability still requires implementation and production verification.
- macOS distribution signing, notarization, distributable packaging, and signed-updater evidence remain later release gates, not requirements for the first local-development milestone or capabilities established by local ad-hoc Proofs.

## Non-Goals

- Adding implementation work to Frozen Specs 00001 or 00002.
- Treating r013 HTML/CSS/JavaScript as production code or research Proofs as production verification.
- Codex plugin or `config.toml` import compatibility.
- Public C4OS marketplace governance or launch.
- Cross-runtime or cross-environment native-session migration.
- Full-screen terminal applications, password-entry terminal flows, browser sub-tabs, multiple simultaneous reply targets, or detached Chat windows.
- Windows or Linux implementation and release claims.

## Accepted Sources

- [C4OS Context index](../../context/index.md).
- [Runtime and session architecture](../../context/runtime-session-architecture.md).
- [Usability and interface contract](../../context/usability-and-interface.md) and its task-specific Reference Files.
- `wireframes/r013-capability-aware-chat/` as accepted product intent and human-reviewable behavior, not implementation evidence.

## Provenance Sources

- [Research Spec 00001](../00001-c4os-ai-harness-research/index.md) for architecture, adapter, model-capability, policy, environment, and Proof detail.
- [Research Spec 00002](../00002-platform-visual-theme-feasibility/index.md) for macOS feasibility and Proof detail.

## Deliverable Standard

This package is ready to Freeze only when its scope is accepted, every material implementation Gap is answered or explicitly deferred, required selection research is complete, any required pre-Freeze Proof is closed, and no requirement conflicts with Context. After Freeze, engineering tasks may be sequenced, but the delivery target is the complete contract without intermediate product-review slice gates. The implementation agent may resolve bounded technical blockers from current research and record its decisions without weakening accepted behavior or security. Progress, verification notes, and final human acceptance records belong under `tasks/`.
