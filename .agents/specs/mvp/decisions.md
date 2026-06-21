# MVP Decisions

Status: ready-for-review
Updated: 2026-06-21

## Decisions

### DEC-001: Full r04 Scope Is MVP

Source: `.agents/specs/research/research-freeze.md`, `.agents/specs/research/checkpoints.md`

The MVP preserves the full documented/r04 product slice unless a later accepted
MVP-spec decision explicitly moves an item out. Checkpoints are implementation
progress gates only.

### DEC-002: Folder-First Trust Gate

Source: `.agents/specs/research/decisions.md`

Prompting and local agent workflows are disabled until a trusted folder is
available.

### DEC-003: Product Model Is App-Owned

Source: `.agents/specs/research/decisions.md`

C4OS owns workspaces, projects, sessions, runs, approvals, artifacts, providers,
files, Browser records, terminal records, extension records, settings, and
persistence.

### DEC-004: OpenCode First Behind Adapter

Source: `.agents/specs/research/poc/results.md`, `.agents/specs/research/decisions.md`

OpenCode is the first runtime backend behind a thin C4OS-owned adapter. Pi
remains a later adapter target.

### DEC-005: Secure Provider BYOK

Source: `.agents/context/constraints.md`, `.agents/specs/research/requirements.md`

Provider support starts with OpenAI-compatible profiles and raw keys in OS
keychain or equivalent secure storage.

### DEC-006: Browser Is User-Owned Desktop Surface

Source: `docs/adr/0001-browser-and-agent-authority.md`

Browser is MVP as a user-owned desktop Browser with project-scoped profile,
local file browsing, public browsing, request-scoped agent browsing, logged-in
use when clearly requested, and recorded agent Browser actions. Downloads are
not MVP.

### DEC-007: Terminal Is Backend-Owned

Source: `.agents/specs/research/decisions.md`, `.agents/specs/research/poc/results.md`

The Terminal tab includes separate user terminal and agent-owned command
terminal surfaces. Renderer shell spawn is rejected for MVP terminal behavior.

### DEC-008: Deterministic Command Allowlist

Source: `.agents/specs/research/decisions.md`

Agent-owned read-only commands use app-bundled exact-pattern allowlist rules
that workspace config can override or extend.

### DEC-009: Extension Impact Requires Enablement

Source: `docs/adr/0002-extension-enablements-and-prompt-tags.md`

Plugins, skills, and MCP servers may be installed or connected in MVP, but they
can affect runtime execution only after explicit per-extension enablement.

### DEC-010: Prompt Extension Tags

Source: `docs/adr/0002-extension-enablements-and-prompt-tags.md`

Chat prompt syntax uses `$skill`, `@plugin`, and `^mcp`. MCP invocation is
explicit only; skills and plugins may also use enabled routing.

### DEC-011: r04 Is Behavioral Handoff

Source: `wireframes/ui-handoff-spec.md`

r04 defines behavior, layout, state, and interaction boundaries. Placeholder
data, sample copy, and grayscale styling are illustrative unless separately
promoted.

### DEC-012: Production Paths Are Fixed

Source: `.agents/AGENTS.md`, `.agents/specs/research/research-freeze.md`

Production implementation belongs in `backend/`, `frontend/`, and
`tests/server/`; `src-tauri/` is not used.

