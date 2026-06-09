# Objective

Determine whether MCP should be included in MVP and, if so, whether only local stdio MCP is acceptable.

# Context

MCP is recommended as a default integration standard, but remote MCP introduces data exfiltration, authorization, prompt-injection, and policy bypass risks.

# Questions To Answer

 - Which MCP spec version should be the baseline?
 - Should MVP allow remote MCP, local stdio MCP only, or no MCP?
 - How are MCP roots scoped to approved project folders?
 - Can MCP tools route through the same approval gateway?
 - Are MCP resources and prompts ever allowed into model context automatically?
 - How are MCP sampling requests approved?
 - What trust levels are needed for MCP servers?

# Hypothesis

MCP should not be in MVP unless tool calls and roots can be forced through the app policy model. Remote MCP likely belongs after MVP.

# Investigation Plan

 - Review MCP spec requirements for tools, roots, resources, prompts, and sampling.
 - Map MCP capabilities to the policy gateway model from SPIKE-002.
 - Threat-model local stdio MCP and remote MCP separately.
 - Identify the smallest safe MCP subset.
 - Document explicit no-go conditions.

# Success Criteria

 - MVP MCP scope is recommended: none, local-only, or remote-capable.
 - MCP root, tool, resource, prompt, and sampling policies are defined at a research level.
 - Risks and no-go conditions are documented.

# Decisions Unlocked

 - ADR-011: MCP Integration Strategy.
 - ADR-004: Policy Enforcement Authority.
 - ADR-005: Standards-First Interoperability.

# Estimated Effort

2 to 4 engineering days.

