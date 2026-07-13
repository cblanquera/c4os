# Plugin Settings And Lifecycle Detailed Decisions

### DEC-014: 2026-07-02: Plugin Sensitive Settings Storage

Source: 2026-07-02 grill intake, "Plugin Sensitive Settings Storage".

  - Where should plugin settings marked sensitive be stored?: Use the
    recommended secure secret storage/keychain rule
  - Accepted rule: Plugin settings marked `sensitive` are stored in
    C4OS-managed secure secret storage/keychain keyed by plugin id and setting
    key. `config.toml` and user config store only references or redacted
    placeholders.
  - Access boundary: Plugin views receive redacted display values. Plugins do
    not receive raw sensitive values by default; they request secret use
    through C4OS-governed tool or service calls.
  - Redaction boundary: Settings, thread context, Chat Debug, logs, and
    inspectable tool state never display or persist raw sensitive setting
    values.

### DEC-015: Backend App Architect Resolution - Plugin Standards And Lifecycle

Source: 2026-07-02 user architect profile in active chat.

  - Standards precedence: When plugin, skill, marketplace, or manifest behavior
    is uncertain, C4OS checks OpenAI/Codex conventions first,
    Claude/Anthropic conventions second, and broader open standards such as MCP
    third. Any local C4OS deviation must be explicit in `agents/c4os.yaml`.
  - Manifest split: Codex-compatible metadata remains in `plugin.json` or the
    current Codex-standard equivalent. C4OS app-shell metadata lives in
    `agents/c4os.yaml`. C4OS must not require standards-owned metadata to be
    duplicated into C4OS-only files.
  - Marketplace safety: Marketplace installation is metadata-first. C4OS reads
    package identity, versions, manifests, settings schema, dependencies, and
    declared capabilities before loading instructions, launching services, or
    granting runtime access.
  - Memory/RAM lifecycle: Plugin views are lightweight per-chat state
    instances. Heavy services are lazy, shared at app/user/workspace/project
    scope, observable, and shut down on idle, disable, uninstall, close, or app
    exit.
  - Windows compatibility: Plugin cache, config, icon paths, service launch,
    trash/recycle hooks, and secret storage must use platform adapters. Specs
    may name macOS/Linux paths only as examples, not as portable contracts.
  - Maintainability: Generated plugin loader/settings code must isolate
    parsing, validation, persistence, lifecycle, service supervision, and UI
    rendering into separate modules with documented state transitions and
    repair reasons.

### DEC-016: 2026-07-02 POC Batch - Plugin Settings And Lifecycle

Source: `proofs/plugin-settings-renderer/`,
`proofs/codex-marketplace-install-cache/`,
`proofs/plugin-svg-sanitization/`, and
`proofs/plugin-lifecycle-pending-restart-and-service-scope/`.

  - Promote schema-rendered plugin settings, redacted sensitive storage, simple
    `visibleWhen`, unknown-key warnings, and reserved-key validation as feasible.
  - Promote metadata-first marketplace cache install, uninstall cache removal,
    and reinstall from source as feasible.
  - Promote static sanitized SVG with fallback icon as feasible.
  - Promote pending-restart backend registration, visible dependency states,
    and narrow shared heavy-service lifecycle as feasible.
  - These are POC decisions only. They do not freeze this spec or create
    implementation progress items.

### DEC-017: 2026-07-02 Approved Batch 2 Wireframes - Plugin Settings

Source: `wireframes/r05-final-implementation/review-round-07.md`,
`wireframes/r05-final-implementation/qa/notes.md`, and approved user review on
2026-07-02.

  - Settings > Plugins keeps the r04-style Plugins page identity and source
    filter behavior. The Built by C4OS source menu shows the current source,
    a separator, and `+ Add Marketplace`.
  - The Built by C4OS marketplace source lists the pending-spec C4OS plugins:
    File system, File editor, Terminal, Chat Debug, and Browser.
  - Advanced settings routes to the plugin detail settings renderer, not to a
    separate marketplace-only surface. The detail view shows schema-rendered
    fields, including input, number, switch, select, sensitive redaction,
    default-only values, panel placement, and icon order.
  - Repair states and tool policy appear below the rendered form on plugin
    detail. Search and source controls do not appear on plugin detail.
  - The approved Settings states preserve visible dependency-blocked,
    incompatible, pending-restart, repairable config, icon fallback, and
    uninstall data-retention prompts.
  - This decision records approved wireframe behavior only. It does not freeze
    this spec or create implementation progress items.
