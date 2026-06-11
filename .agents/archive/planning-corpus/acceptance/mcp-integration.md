# Overview

MCP Integration is excluded from MVP. Acceptance verifies that MCP cannot be configured or invoked in the MVP build.

# Success Criteria

Requirement: MCP is unavailable in MVP.
Expected Result: Users cannot add MCP servers or invoke MCP tools.

Requirement: No MCP tool bypass exists.
Expected Result: Runtime activity contains no MCP calls in MVP validation.

# Functional Acceptance Criteria

Given the MVP build
When the user opens settings
Then no MCP server configuration workflow is available.

Given a project includes MCP configuration files
When the project is registered
Then the app does not start MCP servers or expose MCP tools.

# Security Acceptance Criteria

Given a remote MCP endpoint is present in project files
When the project is opened
Then the app does not connect to the endpoint.

Given a local stdio MCP server is declared in project files
When the project is opened
Then the app does not launch that server.

# Performance Acceptance Criteria

Requirement: MCP exclusion adds no runtime or startup overhead.
Expected Result: No MCP health checks or tool-listing calls occur in MVP.

# User Experience Acceptance Criteria

Requirement: MVP users are not asked to understand MCP.
Expected Result: MCP does not appear as an enabled navigation item or required setup step.

# Failure Conditions

 - Any MCP server starts in MVP.
 - Any remote MCP connection occurs in MVP.
 - Any MCP tool appears in the tool catalog.

# Out Of Scope

 - Local stdio MCP.
 - Remote MCP.
 - MCP resources, prompts, roots, and sampling.
 - MCP authorization flows.

