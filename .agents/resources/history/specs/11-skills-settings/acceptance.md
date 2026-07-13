# Skills Settings Acceptance

Status: proposed

| ID | Acceptance Criteria |
| --- | --- |
| AC-001 | $ suggestions include only enabled valid skills. |
| AC-002 | Invalid skills have visible repair reasons without polluting prompt tags. |
| AC-003 | Skill discovery can list source, status, name, description, validity, parent plugin, and repair reason without loading full skill instructions. |
| AC-004 | Source precedence and customization behavior are defined for bundled, user-global, plugin-provided, and separately promoted project-local skills. |
| AC-005 | Disabled, dependency-blocked, duplicate, unreadable, source-unavailable, or parent-plugin-disabled skills do not enter runtime context or `$` suggestions. |
| AC-006 | Settings copy describes skills as reusable instructions/capabilities for broad workflows, not only code tasks. |
