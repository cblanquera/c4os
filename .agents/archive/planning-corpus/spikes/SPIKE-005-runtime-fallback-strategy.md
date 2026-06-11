# Objective

Define the accepted decision order if direct OpenCode integration fails the MVP runtime-control gate.

# Context

The current corpus says a wrapper, proxy, fork, runtime replacement, or MVP scope reduction must be chosen if direct OpenCode cannot provide mandatory control. FINDING-014 notes that the fallback order is not yet accepted, so a failed runtime spike could reopen broad architecture debate instead of producing a bounded decision.

# Related Findings

 - FINDING-014 What Is The Accepted Runtime Path If Direct OpenCode Fails?
 - FINDING-001 OpenCode Runtime Control Is An Unproven MVP Gate.
 - FINDING-002 Backend Approval Gateway Depends On Runtime Cooperation.
 - FINDING-004 OpenRouter-Only Model Routing Needs Runtime-Level Verification.
 - FINDING-005 Runtime-Native Instruction Loading Can Contradict AGENTS.md Display-Only Scope.

# Questions To Answer

 - What is the fallback decision order after direct OpenCode fails a mandatory MVP gate?
 - Which failed gates favor adapter hardening, tool proxying, OpenCode fork, runtime replacement, or MVP scope reduction?
 - What criteria distinguish an acceptable wrapper/proxy from unsupported terminal scraping or post-execution observation?
 - When is a fork acceptable despite merge debt?
 - When is runtime replacement less risky than preserving OpenCode compatibility?
 - Which MVP capabilities could be reduced without invalidating the product thesis?
 - Which MVP capabilities are non-negotiable because they validate trust, controllability, or local coding usefulness?
 - What evidence is required before choosing a fallback path?
 - Which fallback paths preserve OpenRouter-only routing, backend Approval Gateway authority, app-owned records, and `AGENTS.md` display-only scope?

# Assumptions Being Validated

 - The MVP can avoid open-ended architecture churn by predefining fallback order.
 - Not all OpenCode failures have the same remedy.
 - File-write, shell, Git, model-routing, stop, and instruction-loading failures may require different fallback paths.
 - Scope reduction is acceptable only if the remaining MVP still validates the desktop coding control-center thesis.

# Investigation Plan

 - Use SPIKE-001, SPIKE-002, and SPIKE-004 evidence to classify possible OpenCode failure modes.
 - Define fallback options: adapter constraint, tool proxy, runtime wrapper, OpenCode fork, alternate runtime, direct provider mini-runtime, or MVP scope reduction.
 - Score each option against approval authority, model routing, instruction control, session persistence, stop behavior, engineering cost, maintenance risk, and MVP thesis preservation.
 - Define non-negotiable MVP gates and reducible MVP capabilities.
 - Produce a decision tree that maps failed gates to the recommended next path.
 - Document when the project must pause implementation planning rather than accept a weakened safety model.

# Success Criteria

 - A fallback decision tree exists before Phase 1 implementation planning.
 - Mandatory gates and reducible features are clearly separated.
 - UI-only approvals, post-execution audit, terminal scraping, and best-effort observation are rejected as fallback strategies.
 - Each fallback option has explicit entry criteria and rejection criteria.
 - The selected fallback order preserves backend policy authority and user-visible honesty.
 - The team can respond to a failed OpenCode spike with a bounded decision.

# Deliverables

 - Runtime fallback decision tree.
 - Failure-mode classification table.
 - Fallback option scoring notes.
 - Non-negotiable MVP gate list.
 - Reducible MVP capability list.
 - Recommendation for ADR-003 fallback language.

# ADRs Impacted

 - ADR-003 Agent Runtime Strategy.
 - ADR-004 Policy Enforcement Authority.
 - ADR-008 Unified Tool Invocation And Ledger.
 - ADR-009 Permission And Approval Model.
 - ADR-019 Model Provider Strategy.

# Decisions Unlocked

 - What happens if direct OpenCode fails Phase 0 validation.
 - Whether to pursue wrapper/proxy, fork, alternate runtime, direct provider mini-runtime, or MVP scope reduction.
 - Whether implementation planning can continue after a failed direct-runtime gate.

# Failure Conditions

 - The fallback order remains undefined after runtime-control evidence is available.
 - The accepted fallback depends on UI-only approvals, terminal scraping, post-execution audit, or best-effort observation.
 - The fallback path weakens backend Approval Gateway authority without an explicit ADR change.
 - The fallback path preserves OpenCode at the cost of hidden provider/model or instruction behavior.
 - Scope reduction removes the trust, persistence, controllability, or local coding usefulness needed to validate the MVP thesis.
