# Objective

Identify the single authoritative policy enforcement model for local tools, runtime tools, MCP tools, browser tools, and plugin capabilities.

# Context

The specs say the Rust backend owns policy enforcement while OpenCode performs permission-aware tool use. Reviews flag this as a split-brain risk.

# Questions To Answer

 - Which layer can be the mandatory enforcement point?
 - Can OpenCode execute any tool before the app approves it?
 - Can all tool calls route through a backend policy gateway?
 - How are conflicts between runtime policy and app policy resolved?
 - Where are approval decisions stored?
 - How are denied actions represented to the runtime and model?

# Hypothesis

The app needs a single backend-owned policy gateway for MVP. If runtime tools cannot route through it, the architecture is not safe enough.

# Investigation Plan

 - Use SPIKE-001 findings to list all runtime execution paths.
 - Map file, shell, Git, MCP, browser, artifact, and plugin actions to possible enforcement points.
 - Identify bypass paths.
 - Compare backend-authoritative, runtime-authoritative, and layered enforcement models.
 - Define minimum evidence needed to trust each model.

# Success Criteria

 - One recommended enforcement authority is documented.
 - Bypass paths are identified.
 - Conflict behavior is defined.
 - A no-go condition is documented if pre-execution enforcement is impossible.

# Decisions Unlocked

 - ADR-004: Policy Enforcement Authority.
 - ADR-009: Permission And Approval Model.
 - ADR-010: Filesystem And Shell Access Defaults.
 - ADR-016: Plugin Trust And Execution.

# Estimated Effort

2 to 4 engineering days after SPIKE-001.

