# Overview

Settings and Configuration cover the minimum user-configurable values needed for MVP validation.

# Success Criteria

Requirement: Users can configure OpenRouter credentials and default model.
Expected Result: A session can start after settings are saved.

Requirement: Settings avoid non-MVP concepts.
Expected Result: Plugin, MCP, marketplace, and enterprise settings are not exposed in MVP.

# Functional Acceptance Criteria

Given the user opens settings
When no OpenRouter key is configured
Then the app shows OpenRouter setup as incomplete.

Given the user saves a valid OpenRouter configuration
When the user returns to settings
Then the provider status is shown as ready without displaying the raw key.

Given the user selects a default model
When a new session starts
Then the session uses that selected model path.

Given a session has started
When the user opens settings
Then changes to the default model apply only to future sessions.

# Security Acceptance Criteria

Given settings are persisted
When the local metadata store is inspected
Then raw provider secrets are not present.

Given a non-MVP capability has settings in future plans
When the MVP build is used
Then those settings are absent or disabled.

Given the user opens MVP settings
When approval behavior is displayed
Then file-write batch caps, preview-size caps, approval thresholds, and advanced safety settings are not configurable.

# Performance Acceptance Criteria

Requirement: Opening settings completes within 2 seconds on validation machines.
Expected Result: Configuration is not a workflow bottleneck.

# User Experience Acceptance Criteria

Given provider setup is incomplete
When the user tries to start a session
Then the app directs the user to the required setting.

# Failure Conditions

 - Raw credentials are displayed after save.
 - A session starts with missing provider configuration.
 - Direct provider settings appear in MVP.
 - Settings switch the model for a running session.
 - Settings expose approval thresholds, file-write batch caps, preview-size caps, or advanced safety controls.
 - Non-MVP settings imply unsupported features are available.

# Out Of Scope

 - Multiple provider profiles.
 - Mid-session model switching.
 - Per-agent model overrides.
 - Budget controls.
 - User-configurable approval thresholds.
 - Per-project approval caps.
 - Advanced safety settings.
 - Direct provider configuration.
 - Local model provider configuration.
 - Team settings.
 - Enterprise policy settings.
 - Plugin and MCP settings.
 - Marketplace settings.
