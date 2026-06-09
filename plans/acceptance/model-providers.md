# Overview

Model Providers covers the MVP provider model at a product level. MVP uses OpenRouter only.

# Success Criteria

Requirement: MVP has one supported provider path.
Expected Result: Users configure OpenRouter and can start a model-backed session.

Requirement: Unsupported providers are not exposed as MVP options.
Expected Result: Users cannot configure direct provider keys in MVP.

# Functional Acceptance Criteria

Given the MVP build
When the user opens provider settings
Then OpenRouter is the only configurable provider.

Given provider setup is incomplete
When the user starts a session
Then the app blocks the session and explains that provider configuration is required.

# Security Acceptance Criteria

Given a provider key is entered
When the app stores it
Then the key is stored in OS keychain or encrypted local secret storage.

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

# Failure Conditions

 - A session starts without provider configuration.
 - Provider secrets are stored in plaintext application metadata.
 - Direct provider settings appear as MVP-supported options.

# Out Of Scope

 - Direct OpenAI, Anthropic, Google, or local model provider setup.
 - Provider fallback routing.
 - Advanced provider comparison.
 - Organization-level provider policies.

