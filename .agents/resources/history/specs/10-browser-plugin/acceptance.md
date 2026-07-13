# Browser Plugin Acceptance

Status: proposed

| ID | Acceptance Criteria |
| --- | --- |
| AC-001 | Multiple annotations can attach to one prompt as evidence bundle records. |
| AC-002 | Q036 is recorded as corrected/superseded by Q036A. |
| AC-003 | Browser state follows per-chat plugin instance behavior. |
| AC-004 | Browser navigation/action results created without a visible Browser plugin view are stored as app-owned per-chat tool result state and hydrate when compatible Browser views are later opened. Multiple compatible Browser views hydrate the same source state while keeping view-local UI state separate. |
| AC-005 | Browser events and attachment records are structured enough for prompt interactions, Chat Debug, provider adapters, and compatible Browser views without scraping UI text. |
| AC-006 | The spec documents Browser platform/security assumptions for macOS, Linux, and Windows or marks untested Windows behavior as a validation need. |
| AC-007 | Browser labels and empty states support research/evidence capture for non-coder workflows. |
