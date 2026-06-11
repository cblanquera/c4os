# Risk Analysis

## Runtime Fit Risk

Risk: OpenCode Runtime may not expose enough stable APIs for a desktop GUI, approvals, session persistence, or event streaming.

Mitigation: run Phase 0 prototypes before UI implementation. Keep the runtime adapter narrow so the app can swap runtimes later.

## Security Risk

Risk: local shell, filesystem, MCP, plugins, and browser content create a large attack surface.

Mitigation: enforce least privilege in the backend, maintain an audit ledger, isolate browser content, and require explicit approvals for high-risk operations.

## Plugin Supply Chain Risk

Risk: plugins can bundle malicious instructions, scripts, MCP servers, or assets.

Mitigation: never execute plugins during indexing, require install-then-enable, show requested permissions, support checksums/signatures later, and sandbox script execution through tool policy.

## MCP Prompt Injection Risk

Risk: MCP tools and remote resources may contain instructions that attempt to override user intent or exfiltrate data.

Mitigation: treat MCP descriptions and content as untrusted, scope roots, require approvals, and separate tool metadata from policy.

## Provider Dependence Risk

Risk: OpenRouter routing, model IDs, pricing, and BYOK behavior can change.

Mitigation: cache model metadata with refresh timestamps, expose provider routing in UI when available, and keep a provider abstraction behind OpenCode/OpenRouter.

## Tauri WebView Risk

Risk: system WebView differences may cause inconsistent browser/artifact rendering across platforms.

Mitigation: test target platforms early. Use native rendering libraries or external browser fallback for complex previews if needed.

## Data Loss Risk

Risk: concurrent sessions and worktrees can leave partial edits, orphaned processes, or confusing state.

Mitigation: store process/session/worktree state transactionally, add crash recovery, and never auto-delete dirty worktrees.

## Scope Creep Risk

Risk: a general-purpose workspace can expand into IDE, browser, document editor, and automation platform all at once.

Mitigation: keep MVP anchored to project/session/runtime/artifact/plugin primitives and defer advanced editing/collaboration.

## Interoperability Risk

Risk: standards evolve or differ across Codex, OpenCode, Claude, and MCP implementations.

Mitigation: use adapters, namespaced extensions, and import/export paths instead of hard-forking formats.

## UX Complexity Risk

Risk: multiple agents, sessions, projects, approvals, artifacts, and plugins can overwhelm users.

Mitigation: use progressive disclosure: simple default session view, detailed inspectors on demand, and clear policy explanations.
