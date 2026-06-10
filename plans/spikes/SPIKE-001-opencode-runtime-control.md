# Objective

Determine whether OpenCode can serve as the controllable runtime for the desktop workspace without fragile terminal scraping or unsupported internal coupling.

# Context

The preferred architecture depends on OpenCode as the runtime. Reviews identify this as a make-or-break assumption because the app needs structured events, session control, and pre-execution tool interception.

# Questions To Answer

 - What stable integration surface does OpenCode expose: library API, JSON RPC, CLI protocol, filesystem state, fork, or other?
 - Can the runtime stream structured model, session, tool-call, and tool-result events?
 - Can file, shell, Git, and MCP tool calls be intercepted before execution?
 - Can the app stop, resume, and recover runtime sessions?
 - Can the app cancel an active model stream and terminate app-supervised child processes without deleting app-owned session records or losing partial assistant output status?
 - How are OpenCode runtime session IDs, logs, or persistence files mapped to app-owned canonical session records?
 - Can multiple runtime instances run safely in parallel?
 - Can provider routing and model metadata be surfaced through the runtime?
 - Can OpenCode config override provider or model routing, and can the app detect or prevent it?
 - Does OpenCode automatically load root or nested `AGENTS.md` or other instruction files, and can that behavior be observed, disabled, or disclosed?

# Hypothesis

OpenCode can be used for MVP only if it exposes structured events and reliable pre-execution tool interception through a stable interface.

# Investigation Plan

 - Review OpenCode public documentation and source for runtime, server, CLI, event, and permission APIs.
 - Identify supported integration modes and their stability guarantees.
 - Map MVP-required events and controls to available OpenCode surfaces.
 - Define the runtime-reference mapping for app-owned sessions and tool calls.
 - Test stop behavior for active model streaming, partial assistant output, and app-supervised child processes.
 - Verify the runtime effective model matches the app-owned selected OpenRouter model.
 - Run a minimal proof of structured event capture if documentation is insufficient.
 - Document gaps where the app would need scraping, monkey-patching, or a fork.
 - Document any OpenCode-native instruction loading separately from app-owned root `AGENTS.md` display.
 - Test whether invisible runtime-native instruction loading can be disabled or surfaced to the UI.

# Success Criteria

 - A runtime-control matrix exists for every MVP-required event and control.
 - Pre-execution tool interception is confirmed or ruled out.
 - File writes, shell commands, and Git state changes cannot execute before the app-owned Approval Gateway allows or denies them.
 - Session resume and stop behavior are confirmed or ruled out.
 - Stop cancels active runtime/model streaming and terminates app-supervised child processes while preserving app-owned transcript, partial assistant output status, tool history, approvals, and artifacts.
 - Runtime references can be mapped to app-owned session and tool records without making OpenCode the user-facing source of truth.
 - Provider/model overrides from OpenCode config are detected and prevented, or ruled out.
 - OpenCode-native `AGENTS.md` or instruction-file loading behavior is confirmed or ruled out.
 - Any runtime-native instruction loading is observable and disclosed, or disabled.
 - The team can decide whether OpenCode is viable, needs a fork, or should be replaced.

# MVP Gate

If reliable pre-execution interception is unavailable for MVP file writes, shell commands, and Git state changes, OpenCode is not viable as the direct MVP runtime. The project must choose one of these paths before continuing into implementation:

 - Build a wrapper, proxy, or fork that can enforce the Approval Gateway before execution.
 - Replace the runtime strategy.
 - Reduce the MVP to exclude the affected executable capability.

UI-only approval prompts, post-execution audit logs, terminal scraping, or best-effort observation do not satisfy this gate.

If OpenCode invisibly loads root or nested instruction files and the app cannot observe, disclose, or disable that behavior, OpenCode is not viable as the direct MVP runtime. Invisible instruction injection contradicts the accepted app-owned root `AGENTS.md` display-only boundary.

If OpenCode config can override provider or model routing and the app cannot detect or prevent that override, OpenCode is not viable as the direct MVP runtime. The app must not show one selected OpenRouter model while the runtime uses another provider or model.

If OpenCode cannot reliably cancel an active run and terminate app-supervised child processes while preserving app-owned session records and partial assistant output status, OpenCode is not viable as the direct MVP runtime until an adapter, wrapper, fork, runtime replacement, or scope reduction resolves stop behavior.

# Decisions Unlocked

 - ADR-003: Agent Runtime Strategy.
 - ADR-004: Policy Enforcement Authority.
 - ADR-008: Unified Tool Invocation And Ledger.
 - MVP runtime feasibility.

# Estimated Effort

3 to 5 engineering days.
