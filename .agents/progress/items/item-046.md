# item-046: TASK-045 Resolve V1 Provider Expansion Scope

## Status

verified

## Objective

Choose and document whether V1 should expand beyond the existing
OpenRouter-only provider path into direct providers, local model providers, or
fallback routing.

## Decision

Accepted `openrouter_only_v1_no_direct_or_local_provider_expansion` on
2026-06-12.

V1 should keep OpenRouter as the only configurable provider path. Direct
provider integrations, local model providers, offline fallback, provider
fallback routing, automatic model switching, BYOK provider subconfiguration,
provider-specific settings, and multi-provider configuration remain deferred.

## Dependencies

- item-004
- item-005
- item-045

## Inputs

- `CONTEXT.md`
- `.agents/archive/planning-corpus/acceptance/model-providers.md`
- `.agents/archive/planning-corpus/acceptance/openrouter-integration.md`
- `.agents/archive/planning-corpus/acceptance/provider-expansion.md`
- `.agents/archive/planning-corpus/adr/019-model-provider-strategy.md`
- `.agents/archive/planning-corpus/decisions/deferred-decisions.md`
- `.agents/archive/planning-corpus/spikes/SPIKE-009-provider-privacy-cost.md`

## Deliverables

- Accepted, revised, or rejected V1 provider expansion tier.
- Updated acceptance criteria for allowed and forbidden provider paths.
- Follow-on implementation item only if V1 provider status or UI needs code
  changes after acceptance.

## Acceptance Criteria

- User explicitly accepts, revises, or rejects the support tier.
- OpenRouter-only remains accepted or a revised provider expansion is scoped.
- Direct providers, local models, offline fallback, provider fallback routing,
  automatic switching, and multi-provider settings are explicitly accepted or
  deferred.
- Existing OpenRouter credential, session model, outage, and disclosure
  behavior remains authoritative unless separately revised.

## Verification

- User accepted `openrouter_only_v1_no_direct_or_local_provider_expansion`.
- App status reports the accepted provider expansion tier.
- `cargo test --manifest-path src-tauri/Cargo.toml app_status_reports_openrouter_only_provider_expansion_tier`.
- `npm test -- tests/backend-command-boundary.test.mjs`.
- `npm test`.
- `npm run build`.
- `cargo test --manifest-path src-tauri/Cargo.toml`.
- `npm run tauri -- build`.

## Follow-On

- No direct-provider or local-model implementation is required because the
  accepted tier keeps V1 OpenRouter-only.
