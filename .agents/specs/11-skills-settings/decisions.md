# Skills Settings Decisions

Status: proposed

### DEC-001: 025: C4OS Grill Question 025 - Prompt Tag Routing

Source: `.agents/references/research/final-implementation-import/grill-session/025-c4os-grill-question-025-prompt-tag-routing.json`

  - What should `@` tags target in v1?: Plugin resources and files, based on enabled plugins
  - What should `/` commands route to in v1?: Agent CLI/runtime commands through the runtime/tool gateway
  - What should `$` tags target in v1?: Skills only

### DEC-002: 026: C4OS Grill Question 026 - Disabled Plugin Prompt Tags

Source: `.agents/references/research/final-implementation-import/grill-session/026-c4os-grill-question-026-disabled-plugin-prompt-tags.json`

  - What happens when a prompt tag references a disabled plugin or plugin resource?: Hide disabled plugin resources completely
  - What happens when the plugin is enabled but dependency-blocked?: Hide dependency-blocked resources

### DEC-003: 037: C4OS Grill Question 037 - Skills Settings Scope

Source: `.agents/references/research/final-implementation-import/grill-session/037-c4os-grill-question-037-skills-settings-scope.json`

  - Which skill creator implementation should be pre-populated?: Bundled Codex-compatible skill creator skill
  - Which skill sources should Settings show in the first pass?: Bundled and user-global skills first
  - Can plugin-provided skills be enabled separately from their parent plugin?: No; plugin-provided skills follow parent plugin enablement
  - What metadata is required before a skill appears in `$` tagging?: Name and enabled status only

### DEC-004: 048: C4OS Grill Question 048 - Skills Customization And Invalid States

Source: `.agents/references/research/final-implementation-import/grill-session/048-c4os-grill-question-048-skills-customization-and-invalid-states.json`

  - What happens when the bundled skill creator skill is customized?: Bundled read-only; customization creates user-global copy
  - Which invalid-skill states should Settings distinguish?: Missing SKILL.md, invalid frontmatter, missing name/description, duplicate name, unreadable, and source unavailable
  - When later promoted, should project-local skills require the FS plugin?: Yes; project-local skills require FS plugin
  - How should invalid skills behave in `$` tagging?: Hide from $ suggestions; show in Settings with reason and repair actions
