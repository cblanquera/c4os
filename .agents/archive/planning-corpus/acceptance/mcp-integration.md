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

# V1 Acceptance Criteria

Status: accepted.

V1 may add `local_stdio_explicit_approval_only` support.

## V1 Functional Acceptance Criteria

Given a selected local Git project
When the user opens MCP settings
Then the app may show local stdio MCP as an explicit setup option and must show
remote MCP as unavailable.

Given a project includes MCP configuration files
When the project is registered or selected
Then the app does not auto-discover, auto-launch, or auto-connect MCP servers
from those files.

Given the user explicitly adds a local stdio MCP server
When the server command is reviewed
Then the app shows command, arguments, working directory, root scope, and
unsupported MCP capabilities before any launch.

Given the user explicitly starts a local stdio MCP server
When launch is requested
Then launch routes through the backend Approval Gateway as local shell/process
execution with filtered environment and project-root containment.

Given a local stdio MCP server exposes tools
When a tool invocation is requested
Then invocation must route through the app-owned tool ledger and approval
policy before execution.

Given a local stdio MCP server exposes resources or prompts
When those items are listed
Then resource and prompt contents are not automatically added to app-owned
model context.

## V1 Security Acceptance Criteria

Given a local stdio MCP server requests roots
When roots are provided
Then roots are limited to the selected project root until a later accepted
scope defines narrower roots.

Given a local stdio MCP server requests sampling or elicitation
When the request is received
Then sampling and elicitation are denied or held for explicit approval; no
automatic model call or user-data disclosure occurs.

Given a remote MCP endpoint is configured
When MCP settings or project files are loaded
Then the app does not connect to the endpoint.

Given a local stdio MCP command targets outside-root paths or secret-deny files
When launch or tool invocation is requested
Then existing project-root and secret-deny policy blocks the action.

## V1 User Experience Acceptance Criteria

Requirement: Users must not mistake V1 support for full MCP compatibility.
Expected Result: MCP UI labels unsupported capabilities such as remote MCP,
authorization flows, automatic project-file startup, automatic resource context
ingestion, automatic prompt use, unapproved tool invocation, and unapproved
sampling as unavailable.

Requirement: MCP launch is deliberate.
Expected Result: No MCP server starts unless the user explicitly starts it and
approves the local command when required.

## V1 Failure Conditions

- Any MCP server starts automatically from project files.
- Any remote MCP connection occurs.
- Any MCP tool bypasses app-owned approval and ledger behavior.
- MCP resources or prompts enter model context automatically.
- MCP sampling triggers a model call without explicit approval.
- MCP roots include paths outside the selected project root.
- The app claims full MCP compatibility.

## V1 Out Of Scope

- Remote MCP transports.
- MCP authorization flows.
- Automatic MCP config import.
- Automatic server startup.
- Project-wide or global MCP trust grants.
- Roots outside the selected project root.
- Automatic resource-to-model-context ingestion.
- Automatic prompt use.
- Automatic or unapproved tool invocation.
- Automatic or unapproved sampling.
- MCP marketplace or registry workflows.
- MCP export, import, or round-trip compatibility.
