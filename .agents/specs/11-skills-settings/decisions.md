# Skills Settings Decisions

Status: proposed

### DEC-001: 025: C4OS Grill Question 025 - Prompt Tag Routing

Source: `.agents/references/research/final-implementation-import/grill-session/025-c4os-grill-question-025-prompt-tag-routing.json`

  - What should `@` tags target?: Plugin resources and files, based on enabled plugins
  - What should `/` commands route to?: Agent CLI/runtime commands through the runtime/tool gateway
  - What should `$` tags target?: Skills only

### DEC-002: 026: C4OS Grill Question 026 - Disabled Plugin Prompt Tags

Source: `.agents/references/research/final-implementation-import/grill-session/026-c4os-grill-question-026-disabled-plugin-prompt-tags.json`

  - What happens when a prompt tag references a disabled plugin or plugin resource?: Hide disabled plugin resources completely
  - What happens when the plugin is enabled but dependency-blocked?: Hide dependency-blocked resources

### DEC-003: 037: C4OS Grill Question 037 - Skills Settings Scope

Source: `.agents/references/research/final-implementation-import/grill-session/037-c4os-grill-question-037-skills-settings-scope.json`

  - Which skill creator implementation should be pre-populated?: Bundled Codex-compatible skill creator skill
  - Which skill sources should Settings show in accepted scope?: Bundled and user-global skills
  - Can plugin-provided skills be enabled separately from their parent plugin?: No; plugin-provided skills follow parent plugin enablement
  - What metadata is required before a skill appears in `$` tagging?: Name and enabled status only

### DEC-004: 048: C4OS Grill Question 048 - Skills Customization And Invalid States

Source: `.agents/references/research/final-implementation-import/grill-session/048-c4os-grill-question-048-skills-customization-and-invalid-states.json`

  - What happens when the bundled skill creator skill is customized?: Bundled read-only; customization creates user-global copy
  - Which invalid-skill states should Settings distinguish?: Missing SKILL.md, invalid frontmatter, missing name/description, duplicate name, unreadable, and source unavailable
  - When separately promoted, should project-local skills require the FS plugin?: Yes; project-local skills require FS plugin
  - How should invalid skills behave in `$` tagging?: Hide from $ suggestions; show in Settings with reason and repair actions

### DEC-005: Backend App Architect Resolution - Skills Settings

Source: 2026-07-02 user architect profile in active chat.

  - Standards precedence: Skill package behavior follows OpenAI/Codex
    `SKILL.md` conventions first when available, Claude/Anthropic instruction
    and memory conventions second, and broader open patterns third. C4OS local
    deviations must be visible in Settings and spec records.
  - Discovery safety: Skill discovery is metadata-first. C4OS reads only the
    minimal metadata needed for listing, validation, status, and `$`
    suggestions before a skill is explicitly selected, enabled, customized, or
    loaded for use.
  - Source model: Bundled skills are read-only. User-global customized copies
    are editable and override by explicit user choice. Plugin-provided skills
    follow parent plugin enablement. Project-local skills are deferred unless
    separately promoted and require FS plugin authority.
  - Supply-chain posture: Invalid, duplicate, unreadable, disabled,
    source-unavailable, dependency-blocked, or parent-plugin-disabled skills do
    not appear in `$` suggestions and cannot affect runtime context.
  - Worker-friendly UX: Skills Settings should describe skills as reusable
    instructions or capabilities for many workflows, not just coding helpers.
  - Maintainability: Skill scanner, metadata parser, validation, source
    precedence, customization copy, prompt suggestion filtering, and repair UI
    are separate modules with documented invalid-state reasons.

### DEC-006: POC Result - Skills Settings Invalid States

Source: `proofs/skills-settings-invalid-states/`

  - Result: Passed with `node --test
    proofs/skills-settings-invalid-states/proof.test.mjs`.
  - Decision: Skills Settings can discover metadata without loading full skill
    instructions, list source/status/validity/repair metadata, keep bundled
    skills read-only, create editable user-global customization copies, hide
    invalid/disabled/blocked skills from `$` suggestions and runtime context,
    and keep repair reasons visible in Settings.
  - Reconciliation: This preserves the `$`/`@`/`/` prompt tag split from
    `proofs/prompt-tag-resolution/`, the plugin-provided-skill parent
    enablement boundary, and the FS authority requirement for separately
    promoted project-local skills.
