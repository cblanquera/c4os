# Objective

Determine whether OpenCode can serve as the controllable runtime for the desktop workspace without fragile terminal scraping or unsupported internal coupling.

# Context

The preferred architecture depends on OpenCode as the runtime. Reviews identify this as a make-or-break assumption because the app needs structured events, session control, and pre-execution tool interception.

# Questions To Answer

 - What stable integration surface does OpenCode expose: library API, JSON RPC, CLI protocol, filesystem state, fork, or other?
 - Can the runtime stream structured model, session, tool-call, and tool-result events?
 - Can file, shell, Git, and MCP tool calls be intercepted before execution?
 - Can the app stop, resume, and recover runtime sessions?
 - Can multiple runtime instances run safely in parallel?
 - Can provider routing and model metadata be surfaced through the runtime?

# Hypothesis

OpenCode can be used for MVP only if it exposes structured events and pre-execution tool interception through a stable interface.

# Investigation Plan

 - Review OpenCode public documentation and source for runtime, server, CLI, event, and permission APIs.
 - Identify supported integration modes and their stability guarantees.
 - Map MVP-required events and controls to available OpenCode surfaces.
 - Run a minimal proof of structured event capture if documentation is insufficient.
 - Document gaps where the app would need scraping, monkey-patching, or a fork.

# Success Criteria

 - A runtime-control matrix exists for every MVP-required event and control.
 - Pre-execution tool interception is confirmed or ruled out.
 - Session resume and stop behavior are confirmed or ruled out.
 - The team can decide whether OpenCode is viable, needs a fork, or should be replaced.

# Decisions Unlocked

 - ADR-003: Agent Runtime Strategy.
 - ADR-004: Policy Enforcement Authority.
 - ADR-008: Unified Tool Invocation And Ledger.
 - MVP runtime feasibility.

# Estimated Effort

3 to 5 engineering days.

