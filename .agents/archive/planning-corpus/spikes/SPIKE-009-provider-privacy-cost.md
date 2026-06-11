# Objective

Validate OpenRouter-only provider privacy and cost visibility requirements for MVP.

# Context

Specs choose OpenRouter as the only MVP provider path. Reviews identify lock-in, routing, pricing, fallback, privacy, and future direct-provider migration risks.

# Questions To Answer

 - Which OS keychain or platform credential API should store the OpenRouter key?
 - What actionable error is shown when keychain storage is unavailable or fails?
 - How is a non-secret credential reference captured at session start?
 - What provider data-retention disclosures are required?
 - What exact context is sent through OpenRouter by default?
 - What context-source summary can be shown for model calls without storing raw prompt payloads?
 - How are OpenRouter routes shown to users when available?
 - What provider and network outage errors need normalized retry messaging?
 - How are costs surfaced before and during sessions?
 - How are stale or unavailable model metadata values labeled?
 - How is the runtime effective model verified against the app-owned selected OpenRouter model?
 - What evidence would justify post-MVP direct provider or local model support?

# Hypothesis

OpenRouter-only is accepted for MVP validation if users understand data flow, credentials are stored locally, and cost/routing behavior is transparent enough.

# Investigation Plan

 - Review OpenRouter BYOK, routing, and model metadata behavior.
 - Identify the minimum provider settings for MVP.
 - Validate keychain or platform credential storage for the OpenRouter key.
 - Validate that OpenRouter key update and revoke can be blocked while a session is running.
 - Define the non-secret credential reference captured by sessions.
 - Verify OpenCode cannot silently override app-owned provider/model settings, or define the adapter constraint needed to prevent it.
 - Verify model-call context excludes whole-repo indexing, hidden background ingestion, automatic app-owned root `AGENTS.md` injection, implicit artifact ingestion, and secret-deny file contents.
 - Verify whether OpenCode can expose context-source categories for model calls without per-call approval or raw prompt export.
 - Verify OpenRouter and network outage behavior, including failure state, preserved session records, and explicit retry.
 - Document provider data flow and privacy disclosures.
 - Define cost visibility needs for single-session MVP.
 - Define source and freshness labeling for model metadata when it is available, stale, or unknown.
 - Define validation signals that would trigger post-MVP direct provider or local model work.

# Success Criteria

 - OpenRouter-only provider path requirements are documented for MVP.
 - OpenRouter keychain storage and failure behavior are documented for MVP.
 - OpenRouter credential mutation timing and session credential-reference behavior are documented for MVP.
 - OpenRouter-bound context contents and exclusions are documented for MVP.
 - Standing disclosure and context-source summary behavior are documented for MVP.
 - Provider and network outage failure/retry behavior is documented for MVP.
 - Runtime effective model verification is documented for MVP.
 - Privacy and data-flow disclosures are defined.
 - Cost visibility and metadata freshness labeling requirements are clear.
 - Stale or unavailable pricing, capability, or route metadata is not treated as a session-start blocker.
 - Direct-provider and local model support remain explicitly post-MVP unless validation rejects OpenRouter-only setup.

# Decisions Unlocked

 - ADR-019: Model Provider Strategy.
 - ADR-023: Telemetry And Privacy.
 - MVP validation criteria.

# Estimated Effort

1 to 3 research days.
