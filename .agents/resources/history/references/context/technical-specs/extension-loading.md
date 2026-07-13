# Extension Discovery And Loading

Status: proposed
Created: 2026-06-26
Updated: 2026-06-26

## Purpose

Capture the current research-backed contract for how C4OS should discover,
record, and later load skills, plugins, and MCP servers. This is not an active
implementation task and must not widen TASK-013. Runtime impact remains
disabled until explicit enablement.

## Sources

- OpenAI Codex skills: `https://developers.openai.com/codex/skills`
- OpenAI Codex plugins: `https://developers.openai.com/codex/plugins/build`
- Claude Code skills: `https://code.claude.com/docs/en/skills`
- Model Context Protocol specification: `https://modelcontextprotocol.io/specification/2025-06-18`

## Findings

### Skills

Skills are commonly modeled as folder-based reusable workflow units. The
portable shape is a skill directory with `SKILL.md` as the entrypoint plus
optional supporting files such as `scripts/`, `references/`, `assets/`,
templates, examples, or nested agent configuration.

`SKILL.md` carries metadata and routing instructions. Codex requires `name` and
`description`; Claude recommends a description in YAML frontmatter and supports
additional frontmatter such as invocation/tool controls. Both ecosystems
support progressive disclosure: list or index skill metadata first, then load
the full skill instructions only when the skill is selected or relevant.

Useful C4OS skill roots should be explicit and scope-aware:

- project-local: `<trusted-root>/.agents/skills/<skill>/SKILL.md`
- workspace-local: `<workspace>/.agents/skills/<skill>/SKILL.md`
- user-local: app-managed user skill directory
- plugin-provided: `<plugin-root>/skills/<skill>/SKILL.md`

### Plugins

Plugins are installable bundles rather than prompt instructions by themselves.
The Codex plugin shape uses `.codex-plugin/plugin.json` as the required
manifest. A plugin can point to bundled `skills/`, `.mcp.json`, `.app.json`,
`hooks/`, and `assets/` from the plugin root.

C4OS should treat plugin install as manifest discovery plus component
registration. Installing a plugin should not automatically enable bundled
skills, hooks, MCP servers, apps, or runtime behavior. Each bundled component
needs an app-owned record with provenance, source path, scope, enabled state,
runtime/tool access, shared-data claims, and audit.

### MCP Servers

MCP servers are explicit tool/resource/prompt endpoints connected by a host.
The protocol separates hosts, clients, and servers and exposes capabilities
such as resources, prompts, and tools over negotiated stateful connections.

C4OS should keep MCP records distinct from skills and plugins. A plugin may
bundle `.mcp.json`, but connecting or enabling that server remains an explicit
C4OS action. MCP invocation stays explicit and must not happen implicitly from
normal prompt text.

## C4OS Contract

Discovery should read metadata only:

- skill path, directory name, `SKILL.md` frontmatter, description, and content
  hash
- plugin manifest fields, bundled component references, source, version, and
  content hash
- MCP server metadata/config references without launching the server

Loading should be deferred:

- Do not read full skill instructions until a user or runtime selection needs
  the skill.
- Do not execute plugin hooks during install or discovery.
- Do not launch MCP servers during discovery.
- Do not expose runtime tools until the extension is explicitly enabled and
  mapped through the C4OS tool gateway.

Records should persist:

- id, kind, display name, summary, source path or package source
- provenance and publisher/source metadata
- workspace/project/user/plugin scope
- content hash or version
- enabled state
- runtime access and tool access
- shared data claims
- audit/action references
- disabled or unavailable reason, when applicable

## Sequencing

TASK-012 created inert app-owned extension records. TASK-013 should preserve
those records while implementing concurrent session/run isolation and restart/
resume. Extension discovery/loading should become a later prerequisite for
extension enablement/invocation after durable run state and action/audit records
exist.

Do not use this reference to add extension invocation, runtime tool gateway
execution, hook execution, MCP launch, marketplace installation, or broad
approval-policy hardening to TASK-013.
