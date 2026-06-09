# Overview

OpenRouter Integration covers entering, storing, validating, and using OpenRouter credentials for MVP model-backed sessions.

# Success Criteria

Requirement: Users can configure OpenRouter credentials.
Expected Result: A valid configured key allows an agent session to start.

Requirement: Users understand off-device model calls.
Expected Result: Provider setup communicates that prompts and context are sent through OpenRouter.

# Functional Acceptance Criteria

Given a user enters an OpenRouter key
When the key is saved
Then the app reports provider configuration as ready.

Given provider configuration is ready
When the user starts a session
Then the runtime can use the configured provider path.

Given provider authentication fails
When the user starts a session
Then the session does not start and an actionable provider error is shown.

# Security Acceptance Criteria

Given an OpenRouter key is saved
When application metadata is inspected
Then the raw key is not present in SQLite records.

Given provider errors are logged
When logs are persisted
Then the raw key is not included.

# Performance Acceptance Criteria

Requirement: Provider readiness checks complete within 5 seconds when the provider is reachable.
Expected Result: Users receive timely setup feedback.

# User Experience Acceptance Criteria

Given provider setup succeeds
When the user returns to the session screen
Then the provider-ready state is visible.

# Failure Conditions

 - Invalid credentials are accepted as ready.
 - Raw provider key appears in logs, transcript, or database records.
 - Provider failure is shown as an unexplained runtime failure.

# Out Of Scope

 - OpenRouter account management.
 - Cost dashboard.
 - BYOK provider subconfiguration inside OpenRouter.
 - Direct provider fallback.

