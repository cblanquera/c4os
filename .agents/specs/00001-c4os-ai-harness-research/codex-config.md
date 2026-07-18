# OpenAI Codex `config.toml`

Proof status: passed for versioned one-way subset against `codex-cli 0.145.0-alpha.18` on 2026-07-18. See `proofs/codex-compatibility-fixtures/`.

Research date: 2026-07-18

## Configuration locations and precedence

Codex uses layered TOML configuration rather than one authoritative file:

1. CLI flags and `--config` overrides have the highest ordinary precedence.
2. Trusted project files named `.codex/config.toml` apply from the project root toward the current working directory; the closest file wins.
3. A selected profile file overlays the user configuration.
4. User configuration lives at `$CODEX_HOME/config.toml`, normally `~/.codex/config.toml`.
5. System configuration may live at `/etc/codex/config.toml` on Unix.
6. Built-in defaults have the lowest precedence.

Codex skips project-local `.codex/` configuration, hooks, and rules when the project is not trusted. The CLI and IDE extension share these configuration layers.

Project configuration is intentionally unable to override machine-local provider, authentication, notification, profile-selection, telemetry, and host-owned request settings. The current reference specifically ignores keys such as `openai_base_url`, `chatgpt_base_url`, `model_provider`, `model_providers`, `notify`, `profile`, `profiles`, and `otel` when they appear in project-local configuration.

## Profiles

Current profiles are separate files next to the user configuration, named `$CODEX_HOME/<profile-name>.config.toml`, and selected with `--profile <profile-name>`. They contain normal top-level configuration keys and sit above user config but below project and CLI overrides.

Since Codex 0.134.0, the legacy `[profiles.<name>]` tables and top-level `profile = "<name>"` selector in `config.toml` are no longer the supported profile mechanism. Any compatibility reader must be version-aware rather than assuming one stable historical shape.

## Major schema surfaces

The current reference includes more than model defaults:

| Surface | Representative keys and behavior |
| --- | --- |
| Models and providers | `model`, reasoning/compaction settings, `model_provider`, `openai_base_url`, custom `[model_providers.<id>]`, provider headers, environment-key auth, command-backed bearer-token auth, and local OSS providers. |
| Security and approvals | `approval_policy`, granular approval prompt classes, `approvals_reviewer`, `sandbox_mode`, workspace writable roots, sandbox networking, command rules, login-shell behavior, and Windows sandbox selection. |
| Permission profiles | Built-ins `:read-only`, `:workspace`, and `:danger-full-access`, plus custom `[permissions.<name>]` filesystem/environment/network policies selected through `default_permissions`. |
| MCP | Stdio or streamable HTTP servers, commands/URLs, environment and header inputs, OAuth/bearer auth, required/optional startup, timeouts, enabled/disabled tools, and per-tool approval modes. |
| Skills | `skills.config` entries identify skill-folder paths and per-skill enablement. Portable skill content remains in `SKILL.md`; enablement is host configuration. |
| Apps and tools | Per-app enablement, destructive/open-world gates, default and per-tool approval behavior, and tool suggestions. |
| Hooks | Inline lifecycle hooks plus a feature flag. Current command hook events include tool, permission, compaction, session, subagent, prompt, and stop boundaries; parsed prompt/agent handlers are not currently executed. |
| Agents | Named agent roles, role-specific config layers, thread/depth/runtime limits, and interruption behavior. |
| Features | Stable and experimental flags for apps, hooks, multi-agent, memories, plugins/catalogs, goals, networking, shell behavior, and other evolving capabilities. |
| Local state and UI | History persistence, credential-store selection, logs, SQLite state location, shell environment forwarding, notifications, TUI keymaps, status line, themes, file opener, and telemetry. |

The reference is a living schema with stable, experimental, deprecated, and under-development keys. C4OS must pin any importer to tested Codex versions and preserve diagnostics for fields it does not understand.

## Custom providers and credentials

Custom `[model_providers.<id>]` entries describe base URL, wire API, authentication, and optional headers. Provider tokens may come from environment variables or an external command; they should not be copied into C4OS project files. The built-in OpenAI provider can use `openai_base_url` for a proxy without defining a second provider.

Codex separately controls cached CLI credentials through `cli_auth_credentials_store`: file, OS keyring, or automatic selection. File-backed `auth.json` is sensitive state, not configuration to import or display as ordinary TOML.

## User configuration versus managed policy

`config.toml` supplies user, project, profile, or managed defaults. It is not the non-bypassable enterprise policy surface. Administrators use `requirements.toml` to constrain security-sensitive choices such as allowed approval policies, reviewers, sandboxes or permission profiles, web-search modes, MCP identities, marketplaces, apps, plugins, hooks, network destinations, and feature availability.

Managed defaults standardize startup choices but can be changed during a session; requirements constrain what the user may select. C4OS should keep the same conceptual distinction even if it uses different file formats: defaults are not enforcement.

## C4OS implications

- Codex `config.toml` must not become the C4OS runtime-adapter contract. It contains Codex-host behavior that neither Pi nor OpenCode should be forced to emulate.
- If C4OS supports import, use an explicit versioned allowlist. Candidate portable mappings include selected providers, MCP server declarations, skill enablement, model defaults, and narrowly compatible approval preferences.
- Import should be one-way until lossless round-tripping is proven. Re-exporting a partial schema could silently delete or rewrite Codex settings.
- Keep source provenance and show ignored, translated, conflicting, secret-bearing, and unsupported fields before applying changes.
- Never import `auth.json`, inline secret values, command-produced tokens, telemetry routing, notifications, host paths, or managed requirements as ordinary workspace configuration.
- Project-local import must not elevate machine authority. Provider credentials, executable hooks, MCP commands, additional writable roots, and sandbox relaxation require separate review.
- Translate Codex approval and sandbox concepts into C4OS policy only where semantics match. Similar names are not evidence of equivalent enforcement.
- Keep Codex config import independent of runtime choice: a user may select Pi or OpenCode after importing portable settings.

## Sources

- [Config basics](https://learn.chatgpt.com/docs/config-file/config-basic)
- [Configuration reference](https://learn.chatgpt.com/docs/config-file/config-reference#configtoml)
- [Advanced configuration](https://learn.chatgpt.com/docs/config-file/config-advanced)
- [Authentication](https://learn.chatgpt.com/docs/auth#credential-storage)
- [Managed configuration](https://learn.chatgpt.com/docs/enterprise/managed-configuration)
