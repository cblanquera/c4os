# Skills Settings Requirements

Status: proposed

| ID | Requirement |
| --- | --- |
| REQ-001 | Pre-populate a bundled Codex-compatible skill creator skill. |
| REQ-002 | Show bundled and user-global skills. |
| REQ-003 | Plugin-provided skills follow parent plugin enablement and are not enabled separately. |
| REQ-004 | Skills appear in $ tagging when they have name and enabled status. |
| REQ-005 | Bundled skills are read-only; customization creates a user-global copy. |
| REQ-006 | Settings distinguishes missing SKILL.md, invalid frontmatter, missing name/description, duplicate name, unreadable, and source unavailable. |
| REQ-007 | Project-local skills require FS plugin if separately promoted. |
| REQ-008 | Invalid skills hide from $ suggestions and show in Settings with reason and repair actions. |
| REQ-009 | Resolve skill behavior against OpenAI/Codex skill conventions first, Claude/Anthropic instruction/memory conventions second, and broader open patterns third, with explicit C4OS deviations. |
| REQ-010 | Discover skills metadata-first and avoid loading full skill instructions into runtime context until explicit selection, enablement, customization, or use. |
| REQ-011 | Define source precedence for bundled, user-global, plugin-provided, and separately promoted project-local skills. |
| REQ-012 | Keep skill labels and repair actions understandable for general worker workflows. |
