# Runtime And Tool Policy Acceptance

Status: proposed

| ID | Acceptance Criteria |
| --- | --- |
| AC-001 | Tool policy distinguishes user-directed reads, agent-initiated reads, trusted writes, outside writes, terminal, git, network, credentials, and Browser. |
| AC-002 | Runtime terminal tools remain gateway-owned and visible in thread context plus Chat Debug. |
| AC-003 | Attachments are C4OS records that adapters translate or safely degrade per model. |
| AC-004 | Runtime can execute registered view-oriented tools without an enabled or visible compatible plugin view when policy allows, stores app-owned per-chat inspectable result state for later compatible views, and allows multiple compatible plugin views to hydrate the same shared source state without claiming or mutating it outside explicit C4OS-governed actions. |
| AC-005 | Plugin views hydrate from C4OS-owned state and heavy plugin services are never spawned per chat by default; service lifecycle is shared at the narrowest safe scope and remains governed by C4OS policy. |
| AC-006 | Remembered approval rules are keyed by tool id, action/risk category, normalized target scope where applicable, and plugin id for plugin-contributed tools; users can choose session-only or user-global duration, and user-global remembered rules are reviewable/editable/revocable under Settings > Configuration per server tool. |
| AC-007 | Settings > Configuration shows each registered server tool as a separate policy item with an explanation of what that tool does instead of collapsing approval behavior into one global default policy item. |
| AC-008 | Every tool descriptor has stable ID, human title, description, input schema, output schema or explicit no-structured-output reason, behavior annotations, default approval, maximum authority, and capability requirements. |
| AC-009 | Tool-call lifecycle events are sufficient for shell views, plugin views, Chat Debug, runtime adapters, audit records, and provider result streaming without parsing assistant prose. |
| AC-010 | Tool results prefer structured content, include text fallback when needed for model compatibility, and carry redaction/error/attachment metadata. |
| AC-011 | Timeout, cancellation, memory/output cap, and structured failure behavior are defined before implementation. |
