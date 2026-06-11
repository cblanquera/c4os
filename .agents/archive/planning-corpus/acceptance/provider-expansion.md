# Provider Expansion Acceptance Criteria

## Current Decision

Status: accepted on 2026-06-12.

Accepted tier: `openrouter_only_v1_no_direct_or_local_provider_expansion`.

V1 should keep the existing OpenRouter-only provider path. Direct OpenAI,
Anthropic, Google, local model servers, offline model fallback, provider
fallback routing, automatic model switching, BYOK provider subconfiguration
inside the app, provider-specific settings, provider comparison workflows, and
multi-provider configuration remain deferred.

## Acceptance Criteria

- User explicitly accepts, revises, or rejects
  `openrouter_only_v1_no_direct_or_local_provider_expansion`.
- OpenRouter remains the only configurable provider path for V1.
- Direct provider integrations and local model providers remain unavailable.
- Provider fallback, offline fallback, automatic model switching, and
  multi-provider configuration remain unavailable.
- Existing OpenRouter credential storage, credential mutation, model selection,
  outage handling, and disclosure requirements remain authoritative.

## Out Of Scope

- Direct OpenAI, Anthropic, Google, or other provider setup.
- Local model provider setup.
- Offline model fallback.
- Provider fallback routing.
- Automatic model switching.
- Provider comparison or provider-specific settings UI.
- BYOK provider subconfiguration inside the app.
