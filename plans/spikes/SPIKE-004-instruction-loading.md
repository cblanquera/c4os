# Objective

Resolve whether OpenCode-native instruction loading can coexist with the MVP `AGENTS.md` display-only scope without hidden instruction injection or misleading context disclosures.

# Context

The MVP includes root `AGENTS.md` display so users can inspect project guidance. It does not include full `AGENTS.md` compatibility, nested instruction precedence, hidden app-authored instruction layers, project prompt editors, or automatic root `AGENTS.md` injection into model context. The readiness review flags a contradiction if OpenCode automatically loads root or nested instructions outside app control.

# Related Findings

 - FINDING-005 Runtime-Native Instruction Loading Can Contradict AGENTS.md Display-Only Scope.
 - FINDING-001 OpenCode Runtime Control Is An Unproven MVP Gate.
 - FINDING-004 OpenRouter-Only Model Routing Needs Runtime-Level Verification.

# Questions To Answer

 - Which instruction sources does OpenCode load by default: root `AGENTS.md`, nested `AGENTS.md`, config files, user/global instructions, project instructions, runtime defaults, tool descriptions, or other files?
 - When are those instructions loaded relative to session start, model calls, file reads, and tool activity?
 - Can instruction loading be disabled for MVP sessions?
 - If not disabled, can instruction loading be observed with enough detail to disclose effective instruction sources?
 - Are instruction file reads emitted as normal file-read events and logged in tool activity?
 - Do instruction contents enter OpenRouter model context without explicit runtime file reads?
 - Can root `AGENTS.md` display in the app remain display-only if OpenCode independently loads it?
 - How are nested `AGENTS.md` files handled, and can nested precedence remain out of MVP?
 - Can OpenCode config introduce hidden system prompts, provider prompts, or agent/persona instructions?
 - What user-facing disclosure is required if runtime-native instructions cannot be disabled but can be observed?

# Assumptions Being Validated

 - The app can truthfully claim root `AGENTS.md` is display-only unless explicitly read under normal file-read rules.
 - Runtime-native instruction loading can be disabled, observed, or disclosed.
 - MVP does not need instruction precedence UI if hidden instruction behavior is absent or controlled.
 - Model-call context summaries can accurately identify instruction sources if any are included.

# Investigation Plan

 - Review OpenCode documentation and source for instruction discovery, precedence, config prompts, agent/persona defaults, and project/global instruction behavior.
 - Test or inspect root `AGENTS.md`, nested `AGENTS.md`, absent `AGENTS.md`, global config, and project config scenarios.
 - Record whether instruction reads appear as structured file-read/tool events.
 - Record whether instruction contents are included in model context without explicit file reads.
 - Determine whether launch/config isolation can disable or constrain instruction loading for MVP.
 - Define the minimum disclosure if instruction loading is observable but cannot be disabled.
 - Document whether direct OpenCode fails the MVP gate if instruction loading is invisible or uncontrollable.

# Success Criteria

 - All runtime-native instruction sources relevant to MVP are identified.
 - Instruction loading order and trigger conditions are documented.
 - The app can disable instruction loading, or it can observe and disclose effective instruction sources.
 - Root `AGENTS.md` display-only behavior is either preserved or the MVP scope language is updated by decision.
 - Nested `AGENTS.md` and full instruction precedence remain out of MVP unless explicitly accepted by ADR change.
 - Model-call context summaries can truthfully represent instruction-source inclusion or exclusion.

# Deliverables

 - OpenCode instruction-source inventory.
 - Instruction-loading behavior matrix.
 - Evidence for disable, observe, or disclose options.
 - Recommendation for MVP instruction policy.
 - Required updates to acceptance wording if display-only scope cannot hold.

# ADRs Impacted

 - ADR-003 Agent Runtime Strategy.
 - ADR-004 Policy Enforcement Authority.
 - ADR-019 Model Provider Strategy.
 - ADR-013 Root AGENTS.md Display-Only Scope, currently referenced by rollups.

# Decisions Unlocked

 - Whether direct OpenCode can be used without hidden instruction injection.
 - Whether root `AGENTS.md` remains display-only in MVP.
 - Whether instruction-source disclosure is required in session activity or provider context summaries.
 - Whether nested instruction behavior must stay blocked, be surfaced, or be deferred through runtime constraints.

# Failure Conditions

 - OpenCode invisibly loads root or nested instruction files and the app cannot disable, observe, or disclose it.
 - Instruction contents enter model context without file-read visibility or context-source summary.
 - Existing OpenCode config can add hidden instructions, agents, personas, or prompts without detection.
 - The app continues to claim `AGENTS.md` display-only behavior while runtime-native loading contradicts it.
 - Resolving the behavior requires implementing full instruction precedence UI before MVP.
