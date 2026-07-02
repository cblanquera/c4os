# Browser Plugin Decisions

Status: proposed

### DEC-001: 024: C4OS Grill Question 024 - Browser Tool Defaults

Source: `.agents/references/research/final-implementation-import/grill-session/024-c4os-grill-question-024-browser-tool-defaults.json`

  - What should Browser navigation default to?: Allow all Browser navigation and actions
  - What should screenshots and annotations default to?: All screenshots/annotations attach without asking

### DEC-002: 027: C4OS Grill Question 027 - Attachments First Pass

Source: `.agents/references/research/final-implementation-import/grill-session/027-c4os-grill-question-027-attachments-first-pass.json`

  - Which attachment types must work in accepted scope?: Files, Browser screenshots, and Browser annotations
  - How should C4OS handle providers that do not support direct attachments?: Store C4OS attachment records; runtime adapter translates per provider/model

### DEC-003: 036: C4OS Grill Question 036 - Browser Document Preview

Source: `.agents/references/research/final-implementation-import/grill-session/036-c4os-grill-question-036-browser-document-preview.json`

  - Which document preview formats should Browser support?: Other
  - How should document preview be handled?: Other
  - Notes: Looking at Codex, they have a plugin per doc type (pdf naturally supported by browser). Please confirm before asking me this again.

### DEC-004: 036A: C4OS Grill Question 036A - Document Preview Plugin Boundary

Source: `.agents/references/research/final-implementation-import/grill-session/036a-c4os-grill-question-036a-document-preview-plugin-boundary.json`

  - Who should own document preview support in C4OS final implementation?: Document-family plugins own previews; Browser can host rendered output
  - Which document-family plugins should be planned?: Documents and Spreadsheets
  - How should PDF be treated in accepted scope?: Browser-native PDF preview; advanced PDF plugin only under separate accepted scope

### DEC-005: 042: C4OS Grill Question 042 - Browser Capture, Annotation, And State (no-submitted-intake-response-found)

Source: `.agents/references/research/final-implementation-import/grill-session/042-c4os-grill-question-042-browser-capture-annotation-and-state.json`

  - Status: no-submitted-intake-response-found. No submitted Intake response block for Question 042 was found in this chat context. The durable answer appears in Question 042A after the sample annotation evidence.

### DEC-006: 042A: C4OS Grill Question 042A - Browser Annotation Attachment Model

Source: `.agents/references/research/final-implementation-import/grill-session/042a-c4os-grill-question-042a-browser-annotation-attachment-model.json`

  - Should C4OS follow the Codex-style Browser annotation model shown in the sample?: Yes; use Codex-style target outline, marker, comment, and evidence attachment
  - What should a Browser annotation attachment include?: Screenshot, target metadata, marker number, comment, URL, frame, selector/path, and viewport
  - What screenshot scope should annotations attach by default?: Viewport evidence around the selected target
  - What happens to active annotations after send?: Persist on sent prompt attachments; clear active Browser annotations
  - Notes: It's possible to have many annotations.
