# Chat Prompt Interactions Acceptance

Status: proposed

| ID | Acceptance Criteria |
| --- | --- |
| AC-001 | Approval decisions appear in thread context and Chat Debug when applicable. |
| AC-002 | Prompt tags resolve through enabled resources, preserve inline reference form, and support keyboard navigation while the typeahead menu is open. |
| AC-003 | Unsupported model attachments warn visibly and use safe fallback. |
| AC-004 | Approval UI supports remember choices for session-only or user-global duration and displays an applied remembered-rule summary naming the tool, action/risk category, target scope where applicable, and plugin id for plugin-contributed tools. |
| AC-005 | Approval UI routes review/edit/revoke of user-global remembered rules to Settings > Configuration per registered server tool, while thread context and Chat Debug show the relevant decision summary. |
| AC-006 | Prompt tokenization for display is separate from backend authoritative resolution, and disabled or dependency-blocked resources cannot be executed by frontend-only state. |
| AC-007 | Attachment records declare source plugin/surface, target metadata, size/memory cap behavior, provider compatibility, fallback behavior, and redaction status. |
| AC-008 | Approval prompts emit typed events consumed by thread context, Chat Debug, audit, and runtime resume without scraping UI text. |
