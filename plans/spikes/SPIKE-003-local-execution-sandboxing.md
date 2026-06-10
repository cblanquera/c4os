# Objective

Validate the minimum viable safety model for local filesystem and shell execution.

# Context

The MVP needs useful local coding actions, but shell commands and file writes are high-risk. Current docs decide that MVP shell commands run as the current OS user after approval, may use normal network access, use a backend-filtered environment, use resolved-target containment for symlinks, deny likely secret files with no MVP override, require fresh one-time approval for destructive commands, use manual recovery after bad approved actions, persist bounded shell output summaries, and must not claim strong sandboxing or network isolation. The remaining work is to validate mitigations.

# Questions To Answer

 - What exact redaction matcher and truncation limits are sufficient for MVP shell output summaries?

# Hypothesis

For MVP, current-user shell execution with normal network access, project-root file boundaries, and explicit shell approval may be acceptable only if resolved-target containment, secret-deny behavior, filtered environment handling, redaction, and destructive-command warnings are strict.

# Investigation Plan

 - Document that OS-level sandboxing, containerized execution, and separate-user execution are post-MVP options.
 - Verify path canonicalization and symlink escape test cases.
 - Verify default secret-deny patterns cover common local credential files.
 - Verify destructive command pattern detection against common high-risk examples.
 - Verify filtered-environment behavior against common local build and test workflows.
 - Draft detailed redaction matcher and truncation limits for persisted shell output summaries.
 - Identify commands that cannot be safely classified by pattern alone.
 - Verify approval prompts disclose that approved shell commands are not network-blocked.

# Success Criteria

 - A minimum viable local execution policy is documented.
 - Secret-deny and symlink escape behavior are specified.
 - Filtered-environment behavior and destructive-command handling are specified.
 - Current-user shell execution risks are documented clearly.
 - Normal shell network access risks are documented clearly.

# Decisions Unlocked

 - ADR-010: Filesystem And Shell Access Defaults.
 - ADR-009: Permission And Approval Model.
 - MVP security baseline.

# Estimated Effort

3 to 5 engineering days.
