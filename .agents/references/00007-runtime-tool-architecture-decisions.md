# Runtime Tool Architecture Detailed Decisions

### DEC-017: Backend App Architect Resolution - MCP-Shaped Tool Gateway

Source: 2026-07-02 user architect profile in active chat.

  - Tool shape: C4OS Tauri tools use an MCP-shaped internal contract with
    stable tool ID, title, description, input schema, output schema, behavior
    annotations, default approval policy, maximum authority, capability
    requirements, and structured result envelope.
  - Host boundary: C4OS is the host/gateway. Runtimes and plugins may discover,
    request, or contribute tools, but C4OS owns execution, approval, audit,
    persistence, trusted-root enforcement, result state, and event fanout.
  - Event lifecycle: Every tool call emits typed lifecycle events:
    `tool_call_requested`, `tool_call_policy_evaluated`,
    `tool_call_approved` or `tool_call_rejected`, `tool_call_started`,
    `tool_output_delta`, `tool_call_completed` or `tool_call_failed`, and
    optional `tool_call_cancelled`.
  - Result state: Tool output is persisted as a C4OS result record with
    structured content first, text fallback for model compatibility, resource
    links or attachments where needed, redaction metadata, and provider-adapter
    translation state.
  - Operational controls: Tool calls have request IDs, trace IDs, caller
    identity, target scope, timeout, cancellation, retry policy where safe,
    memory/output caps, and structured error codes.
  - Standards resolution: When gateway semantics are uncertain, prefer
    OpenAI/Codex tool and hosted/local MCP patterns first, then
    Claude/Anthropic agent conventions, then the MCP specification. C4OS local
    deviations must remain documented.

### DEC-018: 2026-07-02 POC Batch - Runtime Tool Policy

Source: `proofs/runtime-tool-discovery-without-plugin-view/`,
`proofs/user-directed-file-access-policy/`,
`proofs/approval-remember-policy/`,
`proofs/pi-runtime-app-layer-proof/`, and
`proofs/model-attachment-adapter/`.

  - Promote runtime discovery/invocation of registered tools without plugin
    views, with C4OS-owned inspectable state for later view hydration.
  - Promote explicit user-directed file policy handling and narrow remembered
    approval rules keyed by tool, risk/action, normalized target scope, and
    plugin id when applicable.
  - Promote the Pi app-layer contract for streaming, tool-call interception,
    approval denial, and resume as feasible pending integration against the
    real runtime package.
  - Promote the OpenAI-compatible attachment adapter direction with visible
    degradation and redacted logs.
  - These are POC decisions only. They do not freeze this spec or create
    implementation progress items.

### DEC-019: 2026-07-02 Approved Batch 2 Wireframes - Tool Policy Settings

Source: `wireframes/r05-final-implementation/review-round-06.md`,
`wireframes/r05-final-implementation/qa/notes.md`, and approved user review on
2026-07-02.

  - Settings > Configuration shows one policy item per registered server tool,
    each with a user-readable explanation plus default and maximum authority.
  - Policy rows use icon-only edit and revoke actions for durable/global
    remembered rule management. Session-only rules are not shown in this
    global Settings surface because they differ per session.
  - Plugin detail may link to tool policy, but tool policy management remains
    under Settings > Configuration and below rendered plugin settings when
    summarized on plugin detail.
  - Config parse errors are represented as a Settings state that keeps the
    last valid config active and blocks saving until the source is repaired or
    restored.
  - This decision records approved wireframe behavior only. It does not freeze
    this spec or create implementation progress items.
