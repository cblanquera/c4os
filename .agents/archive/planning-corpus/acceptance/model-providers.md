# Overview

Model Providers covers the MVP provider model at a product level. MVP uses OpenRouter only.

# Success Criteria

Requirement: MVP has one supported provider path.
Expected Result: Users configure OpenRouter and can start a model-backed session.

Requirement: Unsupported providers are not exposed as MVP options.
Expected Result: Users cannot configure direct provider keys in MVP.

Requirement: MVP uses one model per session.
Expected Result: The selected OpenRouter model is fixed when the session starts and cannot be switched mid-session.

Requirement: App-owned model settings are authoritative.
Expected Result: The runtime effective model matches the selected OpenRouter model for the session.

Requirement: Model metadata is best effort.
Expected Result: Stale or unavailable pricing, capability, or route metadata is labeled and does not block a viable session.

# Functional Acceptance Criteria

Given the MVP build
When the user opens provider settings
Then OpenRouter is the only configurable provider.

Given provider setup is incomplete
When the user starts a session
Then the app blocks the session and explains that provider configuration is required.

Given OpenRouter or network access is unavailable
When the app attempts a model call
Then the call fails closed with a clear provider or network error.

Given a session is running
When the user attempts to update or revoke the OpenRouter key
Then the app blocks the change until the session is stopped or complete.

Given a session has started with a selected model
When the user views model controls
Then no mid-session model switch, fallback route, per-agent override, or budget-control workflow is exposed.

Given a session starts with a selected OpenRouter model
When the runtime is launched
Then OpenCode uses that selected model as the effective session model.

Given existing OpenCode config can override provider or model routing
When the app cannot detect or prevent the override
Then the runtime path is blocked as an MVP Phase 0 failure.

Given provider credentials are valid and the selected OpenRouter model route is viable
When pricing, capability, or route metadata is stale or unavailable
Then the app labels the metadata as stale or unknown and still allows the session to start.

# Security Acceptance Criteria

Given a provider key is entered
When the app stores it
Then the key is stored only in the OS keychain or platform credential store.

Given OS keychain or platform credential storage is unavailable
When the user saves an OpenRouter key
Then provider setup is blocked with an actionable storage error.

Given provider settings are displayed
When the user views them
Then full secret values are not shown.

# Performance Acceptance Criteria

Requirement: Provider configuration validation completes within 5 seconds when network is available.
Expected Result: Setup does not block validation flow unnecessarily.

# User Experience Acceptance Criteria

Given the user configures a provider
When setup is complete
Then the app clearly indicates that model calls leave the local machine.

Given the user configures OpenRouter
When setup is complete
Then the app discloses that prompts and bounded context may be sent through OpenRouter without per-call approval.

# Failure Conditions

 - A session starts without provider configuration.
 - Provider secrets are stored in plaintext application metadata.
 - Provider secrets are stored in project files, env files, shell environments, or a custom encrypted vault.
 - Direct provider settings appear as MVP-supported options.
 - A model is switched mid-session.
 - OpenCode uses a provider or model that differs from the app-owned selected OpenRouter model.
 - Existing OpenCode config overrides provider or model routing without detection.
 - Stale or unavailable model metadata blocks an otherwise viable session.
 - The UI shows stale or unknown pricing, capability, or route metadata as authoritative.
 - Provider setup fails to disclose that bounded model context leaves the machine through OpenRouter.
 - The app requires per-call model-context approval as an MVP workflow.
 - Provider or network outage triggers offline fallback, direct-provider fallback, automatic model switching, queued resend, or synthetic continuation.
 - OpenRouter key update or revoke changes a running session.

# Out Of Scope

 - Direct OpenAI, Anthropic, Google, or local model provider setup.
 - Provider fallback routing.
 - Offline model fallback.
 - Automatic model switching after outage.
 - Queued background resend.
 - App-owned live model catalog sync.
 - Model metadata cache invalidation workflows.
 - Mid-session model switching.
 - Per-agent model overrides.
 - Cost dashboard.
 - Model budget controls.
 - Advanced provider comparison.
 - Organization-level provider policies.
 - Provider secret export/import.
 - Hot key rotation.
 - Per-session credential switching.
 - Mid-call credential retry.
 - Custom encrypted secret vault.
 - Per-call model-context approval.
 - Raw prompt export.
 - Token-by-token context inspection.
 - Editable context composer.
 - Synthetic assistant continuation after provider failure.
