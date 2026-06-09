# Objective

Determine the minimum viable sandbox and safety model for local filesystem and shell execution.

# Context

The MVP needs useful local coding actions, but shell commands and file writes are high-risk. Current docs do not decide whether commands run as the user, a sandboxed process, a container, or a restricted account.

# Questions To Answer

 - Do agent commands run as the user's OS account?
 - Can shell commands be sandboxed on target platforms without breaking developer workflows?
 - What network egress should shell commands have by default?
 - How are symlinks handled under project-root scoping?
 - What files are denied by default?
 - What commands are classified as destructive?
 - What rollback story exists after a bad local action?

# Hypothesis

For MVP, project-root scoping plus explicit shell approval may be acceptable only if symlink handling, deny patterns, and environment filtering are strict.

# Investigation Plan

 - Compare OS-level sandbox options on launch platforms.
 - Evaluate feasibility of containerized or separate-user command execution.
 - Define path canonicalization and symlink test cases.
 - Draft default secret-file deny patterns.
 - Draft destructive command categories.
 - Identify commands that cannot be safely classified by pattern alone.

# Success Criteria

 - A minimum viable local execution policy is documented.
 - Launch-platform sandbox feasibility is known.
 - Secret denylist and symlink behavior are specified.
 - A decision can be made on whether shell execution is allowed in MVP.

# Decisions Unlocked

 - ADR-010: Filesystem And Shell Access Defaults.
 - ADR-009: Permission And Approval Model.
 - MVP security baseline.

# Estimated Effort

3 to 5 engineering days.

