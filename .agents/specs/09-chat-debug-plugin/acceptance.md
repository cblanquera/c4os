# Chat Debug Plugin Acceptance

Status: proposed

| ID | Acceptance Criteria |
| --- | --- |
| AC-001 | Sensitive values, including plugin settings marked `sensitive`, are redacted before display or persistence. |
| AC-002 | Approval decisions appear in both thread context and Chat Debug. |
| AC-003 | Debug records are typed events with structured fields, not parsed prose, and cover runtime/tool/approval/plugin/terminal/attachment/error activity. |
| AC-004 | Debug history retention limits and chat deletion cleanup are defined before implementation. |
| AC-005 | No export surface exists, and redaction tests prove raw sensitive values are absent from stored debug records. |
