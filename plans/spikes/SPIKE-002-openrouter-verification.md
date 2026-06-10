# Objective

Verify that OpenRouter-only model routing can be enforced or observed at runtime so the app does not show one selected provider/model while OpenCode sends prompts through another route.

# Context

ADR-019 finalizes OpenRouter as the only MVP provider gateway. The MVP uses one selected OpenRouter model per session, fixed at session creation, with key storage in the OS keychain or platform credential store. The readiness review flags that this scope reduction is valid only if OpenCode cannot silently override provider/model routing and if runtime-bound context remains within the MVP boundary.

# Related Findings

 - FINDING-004 OpenRouter-Only Model Routing Needs Runtime-Level Verification.
 - FINDING-001 OpenCode Runtime Control Is An Unproven MVP Gate.
 - FINDING-005 Runtime-Native Instruction Loading Can Contradict AGENTS.md Display-Only Scope.

# Questions To Answer

 - How does the app pass OpenRouter provider settings, selected model, and credential reference into OpenCode?
 - How is the runtime effective provider path verified as OpenRouter at session start?
 - How is the runtime effective model verified against the app-owned selected model?
 - Can OpenCode config, environment variables, defaults, retries, fallbacks, or model aliases override app-owned provider/model selection?
 - Can the app prevent or detect hidden provider fallback, model fallback, retry-with-different-model, or direct-provider routing?
 - What non-secret credential reference can be captured for a running session?
 - Can OpenRouter key update/revoke be blocked while a session is running without changing the active runtime credential reference?
 - Can model-call records include bounded context-source summaries without storing raw prompt payloads?
 - Can runtime-bound context exclude whole-repo indexing, hidden background ingestion, automatic app-owned root `AGENTS.md` injection, implicit artifact ingestion, live raw terminal buffers, omitted shell output, and secret-deny file contents?
 - What OpenRouter route, metadata, pricing, capability, source, and freshness information is available and how reliable is it?
 - How should provider or network failures be classified for explicit retry without background resend or synthetic continuation?

# Assumptions Being Validated

 - OpenRouter-only is enforceable as an architecture boundary, not just a UI setting.
 - App-owned model selection remains fixed for a session.
 - A running session keeps its starting credential reference until stopped or complete.
 - Runtime model context can be summarized and bounded without raw prompt export.
 - Missing or stale route/cost metadata can be labeled without blocking otherwise viable sessions.

# Investigation Plan

 - Review OpenCode provider configuration behavior, OpenRouter support, environment-variable precedence, config-file precedence, retry behavior, fallback behavior, and model alias handling.
 - Review OpenRouter API and metadata behavior for route, pricing, capability, freshness, and error surfaces relevant to MVP disclosure.
 - Use minimal probes, if needed, to verify effective provider/model, credential reference stability, config override behavior, and failure behavior.
 - Compare runtime-observed provider/model values with app-owned selected model records across session start, model default change, key update/revoke attempt, provider failure, and retry.
 - Inspect whether runtime context can be constrained to active transcript, selected model/routing metadata, explicit file reads, approved tool results, and bounded shell summaries.
 - Document any unverifiable route, context, or credential behavior as an explicit architecture risk.

# Success Criteria

 - The effective runtime provider path is proven to be OpenRouter or direct OpenCode is rejected for MVP.
 - The effective runtime model is proven to match the app-owned selected model for the session.
 - Existing OpenCode config cannot silently override provider/model routing, or the override is detected and blocked.
 - The session credential reference is stable while a session is running and does not expose raw secret material.
 - OpenRouter key update/revoke cannot affect an active session.
 - Model-call context boundaries and exclusions are verified or documented as unverified blockers.
 - Provider/network errors have a clear fail-closed and explicit-retry classification.
 - Route, pricing, and capability metadata limitations are labeled as informational rather than authoritative.

# Deliverables

 - OpenRouter runtime verification matrix.
 - Provider/model config precedence notes.
 - Credential-reference behavior notes.
 - Model-call context boundary notes.
 - Provider failure and retry classification notes.
 - Metadata availability and freshness notes.
 - Recommendation for whether OpenRouter-only remains acceptable for MVP.

# ADRs Impacted

 - ADR-019 Model Provider Strategy.
 - ADR-003 Agent Runtime Strategy.
 - ADR-004 Policy Enforcement Authority.

# Decisions Unlocked

 - Whether OpenRouter-only can remain the MVP provider strategy.
 - Whether OpenCode requires adapter constraints, config isolation, or rejection as direct runtime.
 - Whether additional provider disclosure, context-source summaries, or metadata labels are required before implementation planning.

# Failure Conditions

 - Runtime model calls can bypass OpenRouter.
 - Runtime model calls can use a different model than the selected session model without detection or prevention.
 - Existing OpenCode config can silently override app-owned provider/model settings.
 - A running session can hot-swap credential references after key update or revoke.
 - Hidden retries, fallback routing, or automatic model switching occur.
 - Whole-repo, secret-deny, unreferenced artifact, root `AGENTS.md`, or raw shell output content can enter model context outside explicit MVP rules.
 - Evidence cannot prove the effective provider/model path.
