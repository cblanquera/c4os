# Plugin System And Settings Management Acceptance

Status: proposed

| ID | Acceptance Criteria |
| --- | --- |
| AC-001 | Settings > Plugins can show enabled, disabled, dependency-blocked, incompatible, pending-restart, and repairable states for typed dependencies covering app plugins, C4OS/preinstalled native modules, heavy services, and required C4OS capabilities/tool identities. |
| AC-002 | Plugin settings schema can render text, textarea, switch, number input, and select fields with label/help, required, placeholder, numeric min/max, enum option labels, sensitive display/storage, and simple `visibleWhen`; unknown keys warn without breaking valid fields. |
| AC-003 | Lifecycle changes are live for UI/settings and restart-gated for native backend registration: newly contributed backend tools stay unavailable until restart, disabled/uninstalled plugin tools are unavailable for new calls immediately, and affected surfaces show restart-required reasons. |
| AC-004 | Uninstall prompts whether to delete plugin-owned user data. |
| AC-005 | Heavy plugin services are not spawned per chat by default; they start lazily, reuse the narrowest safe shared scope, expose health/activity state, and shut down on idle timeout, plugin disable/uninstall, project close, workspace close, or app exit. |
| AC-006 | C4OS preserves `plugin.json` as the Codex compatibility surface and reads `agents/c4os.yaml` only for C4OS app-shell contributions, without requiring Codex metadata duplication or relying on non-standard `plugin.json` extension fields for app-shell behavior. |
| AC-007 | `agents/c4os.yaml` keeps dependencies in a separate top-level manifest section from user-facing `settings`; required missing dependencies block enablement, while optional missing dependencies keep the plugin enabled and visibly hide or degrade only dependent contributions. |
| AC-008 | Shell-reserved settings keys `panel`, `enabled`, and `iconOrder` are accepted only with compatible field types and allowed values; invalid reserved-key declarations disable or repair the affected shell contribution with a visible reason. |
| AC-009 | Sensitive plugin settings persist in secure secret storage/keychain, config files contain only references or redacted placeholders, plugin views show redacted display values, and raw secret use is only available through C4OS-governed tool/service calls. |
| AC-010 | The spec includes a standards-alignment table for plugin manifests, skills, marketplace sources, MCP connections, and C4OS-only metadata, including the accepted local deviation for each. |
| AC-011 | Passive discovery produces metadata and visible status only; service start, hook execution, instruction loading, tool exposure, and runtime access require explicit enablement. |
| AC-012 | Platform-specific cache/config/secret/service/path behaviors identify macOS, Linux, and Windows handling or an explicit deferred validation note. |
| AC-013 | The implementation plan separates loader, parser, validator, lifecycle supervisor, settings renderer, and persistence responsibilities. |
