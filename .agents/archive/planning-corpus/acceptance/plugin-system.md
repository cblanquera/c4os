# Overview

The Plugin System is excluded from MVP. Acceptance verifies that plugins cannot be installed, enabled, or executed.

# Success Criteria

Requirement: Plugin execution is unavailable in MVP.
Expected Result: Plugin scripts, hooks, bundled MCP servers, and plugin assets cannot affect runtime behavior.

# Functional Acceptance Criteria

Given the MVP build
When the user opens settings
Then there is no plugin install or enable workflow.

Given a project contains `.codex-plugin/plugin.json`
When the project is registered
Then the app does not enable, execute, or trust the plugin.

# Security Acceptance Criteria

Given a plugin contains scripts or hooks
When the project is opened
Then no plugin script or hook executes.

Given a plugin declares permissions
When the project is opened
Then those permissions are not granted.

# Performance Acceptance Criteria

Requirement: Plugin indexing does not run in MVP.
Expected Result: Project registration is not affected by plugin package size.

# User Experience Acceptance Criteria

Requirement: The MVP UI does not imply plugin availability.
Expected Result: Users cannot reach plugin installation or plugin settings screens.

# Failure Conditions

 - Plugin code executes.
 - Plugin permissions are granted.
 - Plugin UI appears as an enabled MVP feature.

# Out Of Scope

 - Plugin installation.
 - Plugin enablement.
 - Plugin validation.
 - Plugin scripts and hooks.
 - Plugin permission review.

