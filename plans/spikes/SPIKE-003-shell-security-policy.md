# Objective

Define the concrete MVP shell security policy limits for redaction, truncation, destructive-command classification, filtered environments, and known unclassifiable cases.

# Context

ADR-010 finalizes the broad MVP shell posture: commands run as the current OS user, from the selected project root or approved project subpath, after explicit approval, with normal network access, a backend-filtered environment, no strong sandbox claim, no automatic rollback, and bounded persisted summaries. The unresolved gap is precision. Implementation planning needs exact limits that can be tested.

# Related Findings

 - FINDING-003 Shell Safety Still Lacks Concrete Redaction And Classification Limits.
 - FINDING-002 Backend Approval Gateway Depends On Runtime Cooperation.
 - FINDING-001 OpenCode Runtime Control Is An Unproven MVP Gate.

# Questions To Answer

 - What exact environment variables are kept, stripped, or conditionally kept for MVP shell commands?
 - What exact secret-name and secret-value matchers are required for persisted shell output summaries?
 - What truncation limits apply to stdout, stderr, combined summary size, line length, number of lines, and metadata?
 - What safe reason labels are allowed for redaction, truncation, and output omission?
 - What shell output metadata is forbidden because it could expose redacted values or aid reconstruction?
 - What command patterns are always high-risk and require fresh one-time approval?
 - What examples define recursive deletion, recursive permission changes, disk writes, force Git cleanup/reset, package-manager uninstall/prune, and broad path targeting?
 - What commands are known to be unclassifiable by conservative pattern detection?
 - How should the approval prompt communicate current-user execution, filtered environment, normal network access, manual recovery, and classification uncertainty?
 - When must output summarization fail closed to an output-omitted marker?
 - Which shell safety behaviors must be proven at the runtime boundary rather than only documented in app policy?

# Assumptions Being Validated

 - Current-user shell execution is acceptable for MVP only with precise approval, redaction, truncation, and classification rules.
 - Normal network access is acceptable only if disclosed and approval-gated.
 - Conservative destructive-command detection can reduce risk without claiming perfect shell-expression understanding.
 - Persisted shell records can be useful for debugging without storing raw stdout/stderr.
 - Some commands cannot be safely classified or summarized and must be treated conservatively.

# Investigation Plan

 - Review the security, shell execution, OpenRouter context, and telemetry acceptance criteria for every required shell-output and shell-context boundary.
 - Draft a concrete keep/strip/conditional environment-variable table for MVP.
 - Draft a minimum redaction matcher set covering known secret values, secret-shaped values, common token formats, private keys, credential URLs, cloud credentials, package tokens, and provider API keys.
 - Draft truncation limits for persisted summaries and live-buffer-to-persisted-record transitions.
 - Draft a destructive-command classification table with examples, non-examples, and always-one-time categories.
 - Identify command shapes that cannot be reliably parsed by MVP policy, including shell indirection, encoded scripts, eval-like execution, generated scripts, aliases/functions, command substitutions, broad globs, and nested interpreters.
 - Define fail-closed criteria for unsafe summary generation and uncertain destructive classification.
 - Record how these rules interact with model context: only persisted summaries, output-omitted markers, and safe reason labels may be sent.

# Success Criteria

 - The MVP has exact redaction matcher categories and explicit limitations.
 - The MVP has exact truncation limits and output-omission rules.
 - The MVP has an allowed safe reason-label set.
 - The MVP has an explicit list of forbidden persisted metadata.
 - The MVP has destructive-command examples and high-risk categories that require fresh one-time approval.
 - The MVP has documented unclassifiable command cases and conservative handling.
 - Approval disclosure requirements are concrete enough to test.
 - The policy does not claim sandboxing, network isolation, perfect classification, or automatic rollback.

# Deliverables

 - Shell environment filtering table.
 - Redaction matcher and known-limits table.
 - Truncation and output-omission limits.
 - Safe reason-label list.
 - Forbidden metadata list.
 - Destructive-command classification table.
 - Unclassifiable command examples and required handling.
 - Approval prompt disclosure requirements.

# ADRs Impacted

 - ADR-010 Filesystem And Shell Access Defaults.
 - ADR-009 Permission And Approval Model.
 - ADR-004 Policy Enforcement Authority.
 - ADR-019 Model Provider Strategy.

# Decisions Unlocked

 - Whether shell execution remains in MVP under the accepted current-user, normal-network posture.
 - Which shell policy rules are mandatory Phase 0 gates.
 - Which shell behaviors are explicitly accepted MVP risks.

# Failure Conditions

 - Redaction rules are too vague to test.
 - Truncation limits are absent or allow reconstruction hints.
 - Raw stdout/stderr can be persisted as fallback after summary failure.
 - Known secret values, secret-shaped values, or provider keys can appear in persisted records.
 - Destructive commands can execute under ordinary session allow.
 - Approval prompts imply sandboxing, network isolation, automatic rollback, or perfect command understanding.
 - Shell tool context sent to OpenRouter can include live raw terminal output, omitted raw output, redacted substrings, sensitive raw byte counts, offsets, hashes, or reconstruction metadata.
