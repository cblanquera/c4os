# Objective

Determine whether OpenRouter-only provider support is acceptable for MVP and define provider privacy and cost visibility requirements.

# Context

Specs choose OpenRouter with BYOK as the preferred provider path. Reviews identify lock-in, routing, pricing, fallback, privacy, and direct-provider migration risks.

# Questions To Answer

 - Is OpenRouter mandatory or optional in MVP?
 - Should direct provider fallback be required?
 - Where are BYOK credentials stored and managed?
 - What provider data-retention disclosures are required?
 - How are provider routes and fallbacks shown to users?
 - How are costs surfaced before and during sessions?
 - How is model capability metadata refreshed?

# Hypothesis

OpenRouter-only is acceptable for MVP validation if users understand data flow, credentials are stored locally, and cost/routing behavior is transparent enough.

# Investigation Plan

 - Review OpenRouter BYOK, routing, and model metadata behavior.
 - Identify the minimum provider settings for MVP.
 - Document provider data flow and privacy disclosures.
 - Compare effort of direct provider fallback.
 - Define cost visibility needs for single-session MVP.

# Success Criteria

 - Provider path recommendation is documented for MVP.
 - Privacy and data-flow disclosures are defined.
 - Cost visibility requirements are clear.
 - Direct-provider fallback is classified as MVP, V1, or later.

# Decisions Unlocked

 - ADR-019: Model Provider Strategy.
 - ADR-023: Telemetry And Privacy.
 - MVP validation criteria.

# Estimated Effort

1 to 3 research days.

