# Plugin System Specification

## Goals

Plugins should package repeatable workflow capabilities while preserving interoperability with Codex-compatible plugin and marketplace conventions.

## Plugin Layout

Required:

```text
plugin-name/
  .codex-plugin/
    plugin.json
```

Optional:

```text
plugin-name/
  skills/
  hooks/
  scripts/
  assets/
  .mcp.json
  .app.json
```

Recommendation: require `.codex-plugin/plugin.json` and support Codex-compatible optional folders.

Alternatives considered:
 - Require only MCP manifests: too narrow.
 - Invent a new plugin manifest: easier to tune, worse sharing with Codex ecosystem.

Why selected: Codex-compatible plugin shape already maps to skills, MCP, assets, scripts, and app templates.

## Manifest Requirements

The manifest should include normalized plugin name, version, display metadata, description, authorship, license, capabilities, and compatibility. Non-Codex fields must be namespaced under `x-c4os` to avoid breaking compatibility.

## Plugin Capabilities

Supported capability types:
 - Skills bundled in `skills/`.
 - MCP servers declared in `.mcp.json`.
 - Assets used by skills or artifact generation.
 - Scripts used by skills or runtime helpers.
 - App connector templates in `.app.json`.
 - UI metadata for marketplace display.
 - Optional hooks only if the runtime adapter supports a compatible hook surface.

## Installation Scopes

 - Global user plugins.
 - Project plugins.
 - Team/repository marketplace plugins.

Project-level enablement should override global availability only within that project.

## Permissions

Plugins must declare requested permissions:
 - Filesystem roots or path patterns.
 - Shell commands or command patterns.
 - Network access.
 - MCP servers and tool categories.
 - Secrets needed.
 - Write actions.
 - Artifact types.

Enablement must show requested permissions and allow partial disablement when practical.

## Validation

The app must validate:
 - Manifest exists and parses.
 - Plugin name matches normalized folder name.
 - Skills contain valid `SKILL.md` files.
 - MCP config is syntactically valid.
 - Referenced assets/scripts exist.
 - Marketplace policy fields are valid.
 - No unsupported top-level fields unless namespaced.

## Runtime Loading

Plugins are indexed at startup and when installed. The app must not automatically execute plugin scripts during indexing. Runtime capabilities become active only after the plugin is enabled and policy evaluation permits them.

## Security Recommendation

Treat plugins as untrusted packages until installed and approved.

Alternatives considered:
 - Trust all local plugins: convenient, but unsafe.
 - Ban scripts in plugins: safer, but breaks useful repeatable workflows.

Why selected: explicit permissions and non-executing indexing preserve flexibility without silent execution.
