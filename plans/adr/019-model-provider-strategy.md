# ADR-019: Model Provider Strategy

Status: Provisional.

## Context

The preferred architecture routes model access through OpenRouter with BYOK support. Functional requirements include model switching, provider presets per project and agent, OpenRouter credits, BYOK provider keys, and provider route visibility when available.

The review identifies OpenRouter lock-in risks: routing, pricing, BYOK fees, model slugs, provider behavior, privacy disclosures, and future direct-provider migration.

## Decision

Use OpenRouter as the primary provider gateway for MVP, with BYOK support.

This decision is provisional because direct provider fallback and provider-neutral capability modeling are unresolved.

## Alternatives Considered

 - OpenRouter primary.
 - Direct provider calls.
 - OpenCode provider abstraction.

## Alternatives That Should Be Considered

 - Direct provider fallback for critical providers.
 - Local model provider support.
 - Provider-neutral model capability registry.
 - OpenRouter as optional gateway rather than required gateway.

## Tradeoffs

OpenRouter simplifies multi-provider access, BYOK, routing, and model switching.

It introduces dependency on external routing behavior, model IDs, pricing, and policy changes.

Direct providers reduce intermediary lock-in but increase credential, routing, pricing, and capability complexity.

## Consequences

 - Provider-specific fields should be isolated from canonical model records.
 - Users need visibility into data leaving the machine.
 - Model cost and fallback behavior should be surfaced before and during long-running work.

## Follow-Up Questions

 - Should direct provider calls be supported if OpenRouter is unavailable?
 - Where are BYOK credentials stored and managed?
 - How are provider fallbacks shown to users?
 - How are model costs and budgets enforced?
 - What data-retention disclosures are required per provider?

## ADR Recommendation

Keep this ADR high priority before provider settings and model UX are specified in detail.

