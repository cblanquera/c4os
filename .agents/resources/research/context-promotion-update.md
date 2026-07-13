# Context Promotion Update

Status: accepted
Updated: 2026-06-21
Source: `.agents/references/research/grill-session.md`

## Promoted To Context

- Full documented/r04 scope is MVP unless explicitly moved out later.
- OpenCode is the first runtime backend behind the C4OS-owned adapter; Pi is a
  later adapter target.
- Agent authority is request-scoped: inside-project file reads/writes are
  allowed when implied by the request, while outside-project file access
  requires explicit permission.
- Browser is a user-owned desktop surface with project-scoped profile,
  local-file browsing, public web browsing, request-scoped agent browsing, and
  recorded agent Browser actions.
- Terminal uses separate user terminal and agent-owned command terminal
  surfaces, with deterministic command allowlist, approvals, and audit records.
- Extensions require explicit per-extension enablement before runtime impact.
- Prompt tags are `$skill`, `@plugin`, and `^mcp`.

## Updated Context Files

- `.agents/context/work-orders.md`
- `.agents/context/technical-specs.md`
- `.agents/context/product-brief.md`
- `.agents/references/context/product/terms.md`
- `.agents/context/creative-specs.md`
