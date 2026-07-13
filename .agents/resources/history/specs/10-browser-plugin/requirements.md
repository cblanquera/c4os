# Browser Plugin Requirements

Status: proposed

| ID | Requirement |
| --- | --- |
| REQ-001 | Support Browser navigation, back, forward, refresh, and Browser actions allowed by default. |
| REQ-002 | Screenshots and annotations attach to chat prompt without asking. |
| REQ-003 | Support many Codex-style annotations with target outline, marker, comment, screenshot, target metadata, URL, frame, selector/path, and viewport. |
| REQ-004 | Persist sent prompt annotation attachments and clear active Browser annotations after send. |
| REQ-005 | Document-family plugins own .docx/.xlsx previews; Browser can host rendered output. Browser-native PDF preview comes accepted; advanced PDF plugin is separate. |
| REQ-006 | Browser activation must not create a chat item. |
| REQ-007 | Browser-oriented runtime tool activity can execute without a visible Browser plugin view and must leave app-owned per-chat inspectable Browser state for later compatible views. |
| REQ-008 | Emit typed Browser events for navigation, actions, screenshots, annotations, prompt attachment, clear-after-send, and document-preview handoff. |
| REQ-009 | Include URL, frame, selector/path, viewport, marker/comment, screenshot metadata, source plugin, memory/size limits, redaction status, and provider compatibility in Browser attachment records. |
| REQ-010 | Define platform/security provisions for webview differences, profile isolation, local-file access, and privileged bridge isolation on macOS, Linux, and Windows. |
| REQ-011 | Keep Browser UX useful for general research/evidence workflows, not only coding preview. |
