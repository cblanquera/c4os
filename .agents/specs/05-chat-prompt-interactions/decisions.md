# Chat Prompt Interactions Decisions

Status: proposed

### DEC-001: 020: C4OS Grill Question 020 - App Tool Approval Defaults

Source: `.agents/references/research/final-implementation-import/grill-session/020-c4os-grill-question-020-app-tool-approval-defaults.json`

  - How should C4OS define default approval policy?: Per Tauri tool/app tool with default policy and max authority
  - Which approval categories should exist?: allow, ask, deny, remember
  - Remembered approval UI: When a user chooses `remember`, the approval UI
    offers session-only or user-global duration. The UI summary names the tool,
    action/risk category, target scope when applicable, and plugin id for
    plugin-contributed tools.
  - Review/revoke routing: The approval UI and Chat Debug/thread context can
    show the applied remembered-rule summary, but Settings > Configuration is
    the review/edit/revoke surface. Settings > Configuration lists one policy
    item per registered server tool and explains what each server tool does.
  - Refinement source: 2026-07-02 grill intake, "Approval Remember Rule
    Semantics".

### DEC-002: 025: C4OS Grill Question 025 - Prompt Tag Routing

Source: `.agents/references/research/final-implementation-import/grill-session/025-c4os-grill-question-025-prompt-tag-routing.json`

  - What should `@` tags target?: Plugin resources and files, based on enabled plugins
  - What should `/` commands route to?: Agent CLI/runtime commands through the runtime/tool gateway
  - What should `$` tags target?: Skills only

### DEC-003: 026: C4OS Grill Question 026 - Disabled Plugin Prompt Tags

Source: `.agents/references/research/final-implementation-import/grill-session/026-c4os-grill-question-026-disabled-plugin-prompt-tags.json`

  - What happens when a prompt tag references a disabled plugin or plugin resource?: Hide disabled plugin resources completely
  - What happens when the plugin is enabled but dependency-blocked?: Hide dependency-blocked resources

### DEC-004: 027: C4OS Grill Question 027 - Attachments First Pass

Source: `.agents/references/research/final-implementation-import/grill-session/027-c4os-grill-question-027-attachments-first-pass.json`

  - Which attachment types must work in accepted scope?: Files, Browser screenshots, and Browser annotations
  - How should C4OS handle providers that do not support direct attachments?: Store C4OS attachment records; runtime adapter translates per provider/model

### DEC-005: 028: C4OS Grill Question 028 - Branch Selection

Source: `.agents/references/research/final-implementation-import/grill-session/028-c4os-grill-question-028-branch-selection.json`

  - When should the chat prompt branch button appear?: Only when the active project folder is a Git repository
  - What should Choose/Create Branch do?: Create/select project branch for separate accepted work; existing chat thread branch remains read-only

### DEC-006: 032: C4OS Grill Question 032 - File Explorer Add To Chat

Source: `.agents/references/research/final-implementation-import/grill-session/032-c4os-grill-question-032-file-explorer-add-to-chat.json`

  - What should File Explorer `Add to chat` attach?: should inline paste the same as prompt tagging ie. `@bin.js`
  - What should Copy Path copy?: Absolute filesystem path

### DEC-007: 042A: C4OS Grill Question 042A - Browser Annotation Attachment Model

Source: `.agents/references/research/final-implementation-import/grill-session/042a-c4os-grill-question-042a-browser-annotation-attachment-model.json`

  - Should C4OS follow the Codex-style Browser annotation model shown in the sample?: Yes; use Codex-style target outline, marker, comment, and evidence attachment
  - What should a Browser annotation attachment include?: Screenshot, target metadata, marker number, comment, URL, frame, selector/path, and viewport
  - What screenshot scope should annotations attach by default?: Viewport evidence around the selected target
  - What happens to active annotations after send?: Persist on sent prompt attachments; clear active Browser annotations
  - Notes: It's possible to have many annotations.

### DEC-008: 043: C4OS Grill Question 043 - Chat Debug Redaction And History

Source: `.agents/references/research/final-implementation-import/grill-session/043-c4os-grill-question-043-chat-debug-redaction-and-history.json`

  - Should Chat Debug ever be enabled by default?: Disabled by default everywhere
  - Which Chat Debug values must always be redacted?: Secrets, tokens, raw credentials, provider keys, auth headers, cookies, and sensitive params
  - Should Chat Debug show historical runs?: Current and historical runs for the active chat session
  - Should users be able to export Chat Debug logs?: No export
  - Where should approval decisions appear?: Both thread context and Chat Debug

### DEC-009: 045: C4OS Grill Question 045 - Model Attachment Compatibility

Source: `.agents/references/research/final-implementation-import/grill-session/045-c4os-grill-question-045-model-attachment-compatibility.json`

  - How should C4OS handle model/provider attachment differences?: Store C4OS attachment records; adapters translate or degrade per model
  - What should happen when a selected model cannot consume an attachment directly?: Warn visibly and use best safe adapter fallback
  - Which provider/model path should the attachment proof target?: Current OpenAI-compatible provider path
  - Which attachment types need accepted proof coverage?: Files, Browser screenshots, and Browser annotation attachments

### DEC-010: 049: C4OS Grill Question 049 - App Tool Taxonomy And Pi Proof

Source: `.agents/references/research/final-implementation-import/grill-session/049-c4os-grill-question-049-app-tool-taxonomy-and-pi-proof.json`

  - Which app-tool taxonomy should be frozen before plugin details?: Let each plugin define its own taxonomy independently
  - Which app-tool categories must be represented?: Only plugin-contributed tools
  - Which Pi capabilities are proof-critical?: Prompt execution, streaming, tool-call interception, approval denial, and resume
  - Where should `/` prompt commands route?: Runtime/tool gateway command handling with C4OS and plugin command definitions

### DEC-011: Backend App Architect Resolution - Prompt Interactions

Source: 2026-07-02 user architect profile in active chat.

  - Parsing boundary: `$`, `@`, and `/` are UI affordances backed by C4OS
    resolver records. The frontend may tokenize for display, but authoritative
    resolution, permission checks, disabled-resource filtering, and command
    execution route through backend/gateway services.
  - Worker-friendly language: Prompt UI labels should say Skills, Files,
    Resources, Commands, Approvals, Attachments, and Branch only when the
    concept is actually relevant. Coding-specific labels must not leak into
    non-code workflows.
  - Attachment model: Files, Browser screenshots, Browser annotations, and
    plugin-provided references become C4OS attachment/reference records with
    source plugin, target metadata, size/memory limits, provider compatibility
    status, and redaction policy.
  - Approval UI: Approval prompts show tool title, action, target, risk,
    plugin/source, remembered duration, and plain-language impact. They emit
    typed approval decision events for thread context, Chat Debug, audit, and
    runtime resume.
  - Maintainability: Prompt interaction implementation separates tokenizer,
    resolver, attachment store, approval event contract, provider adapter
    degradation, and view components.

### DEC-012: 2026-07-02 POC Batch - Prompt Interaction Flow

Source: `proofs/approval-ui-flow/`, `proofs/prompt-tag-resolution/`, and
`proofs/attachment-compatibility/`.

  - Promote typed approval decision events for allow, deny, and remember flows.
    Remembered-rule summaries include tool, action/risk category, target
    scope, plugin id when relevant, and session-only or user-global duration.
  - Promote backend-authoritative `$`, `@`, and `/` resolution, with frontend
    tokenization limited to display and disabled or dependency-blocked
    resources hidden from executable resolution.
  - Promote C4OS attachment records for files, Browser screenshots, and Browser
    annotations with source, target metadata, limits, provider compatibility,
    fallback, and redaction fields.
  - The attachment proof depends on the already passed
    `proofs/model-attachment-adapter/` provider-adapter direction and covers
    prompt-level record collection and handoff rather than re-proving provider
    internals.
  - These are POC decisions only. They do not freeze this spec or create
    implementation progress items.
