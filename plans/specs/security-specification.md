# Security Specification

## Security Model

The application must assume that model output, remote web content, MCP tool descriptions, plugins, generated files, and shell output can be malicious or misleading.

## Trust Boundaries

 - Tauri frontend: untrusted UI surface relative to backend capabilities.
 - Tauri Rust backend: trusted policy and execution boundary.
 - OpenCode runtime: semi-trusted execution engine constrained by policy.
 - MCP servers: untrusted unless explicitly trusted by user/admin.
 - Plugins: untrusted until installed and enabled.
 - Project files: user data, not automatically safe instructions.
 - Remote web pages: untrusted content.

## Permission Layers

 - Workspace trust: trusted, restricted, or read-only.
 - Project policy: roots, command patterns, network mode, Git mode.
 - Agent policy: allowed tools, model, skills, subagents.
 - Plugin policy: requested capabilities and user/admin grant.
 - MCP policy: server allowlist, roots, tools, sampling.
 - Tool-call policy: allow, ask, deny.

## Approval UX

Approvals must show:
 - Tool name and provider.
 - Arguments and target resources.
 - Risk category.
 - Policy source.
 - Whether the call reads, writes, executes, sends network data, or uses credentials.
 - Scope of approval.

Approval scopes:
 - Once.
 - Session.
 - Project.
 - Always for this plugin/server/tool pattern.
 - Deny once.
 - Always deny.

## Filesystem Security

 - Default access is project root only.
 - External directory access requires explicit approval.
 - Secret files should be denied by default using configurable patterns.
 - File writes outside approved roots are blocked.
 - Generated artifacts are stored in app-managed artifact directories unless explicitly exported.

## Shell Security

 - Shell execution defaults to `ask`.
 - Destructive commands require explicit approval.
 - Commands run with project working directory and inherited environment filtered by policy.
 - Secrets must not be injected into shell environment unless required and approved.
 - PTY output is logged with truncation controls.

## MCP Security

Follow MCP trust guidance:
 - User consent for data access and operations.
 - Human-in-the-loop tool invocation.
 - Treat tool annotations as untrusted unless the server is trusted.
 - Require approval for sampling requests.
 - Scope roots to approved project folders.

## Plugin Security

 - Indexing must not execute plugin code.
 - Plugin scripts execute only as approved tools.
 - Plugins declare capabilities before enablement.
 - Remote plugin sources should support checksum or signatures in future phases.

## Secret Storage

Store OpenRouter keys, provider BYOK credentials, OAuth tokens, and app secrets in the OS keychain or encrypted vault. SQLite must store references, not raw secret values.

## Desktop Security

Use Tauri capabilities to restrict frontend access. Disable remote web content access to backend commands unless explicitly required. Browser panels must not receive privileged IPC.

Recommendation: security enforcement belongs in Rust backend and runtime policy, not only frontend UI.

Alternatives considered:
 - Frontend-only checks: easy to bypass.
 - Runtime-only checks: misses plugin install, UI browser, and local storage boundaries.

Why selected: defense-in-depth is required for local shell and file access.
