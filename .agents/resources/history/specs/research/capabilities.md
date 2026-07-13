# Research Capabilities

## CAP-001: Folder-First Workspace

Status: proposed
Confidence: imported
MVP: yes
Phase: mvp
Source: `plans/product-brief.md`

Users can open, create, clone, select, and trust local project folders before agent workflows become available.

## CAP-002: Persistent Sessions

Status: proposed
Confidence: imported
MVP: yes
Phase: mvp
Source: `plans/product-brief.md`

Projects support resumable sessions with transcript, run status, provider/model selection, approval state, artifact references, and runtime references.

## CAP-003: Provider And Model Management

Status: proposed
Confidence: imported
MVP: yes
Phase: mvp
Source: `plans/product-brief.md`

Users can configure OpenAI-compatible provider profiles, securely store keys, discover or enter models, and select models per session.

## CAP-004: App-Owned Approval Gateway

Status: proposed
Confidence: imported
MVP: yes
Phase: mvp
Source: `plans/product-brief.md`

Sensitive actions are evaluated by app-owned policy before execution, with visible approval requests, decisions, saved rules, and audit logs.

## CAP-005: File Explorer And Editor

Status: proposed
Confidence: imported
MVP: yes
Phase: mvp
Source: `plans/product-brief.md`

Users can inspect and edit trusted-root files through guarded file browsing, code view, dirty state, save, revert, and later file-management actions.

## CAP-006: Artifacts And Safe Preview

Status: proposed
Confidence: imported
MVP: yes
Phase: mvp
Source: `plans/product-brief.md`

Generated outputs are first-class artifacts with project/session identity and safe viewers, including isolated handling for generated HTML.

## CAP-007: Browser Surface

Status: accepted
Confidence: user-accepted
MVP: yes
Phase: mvp
Source: `plans/product-brief.md`, `.agents/references/research/grill-session.md`

A Browser tab supports user-owned desktop browsing with project-scoped Browser profile, public web access, local file browsing, request-scoped agent browsing, logged-in session use when required by the user's request, and audit records for agent Browser actions.

## CAP-008: Terminal Surface

Status: accepted
Confidence: user-accepted
MVP: yes
Phase: mvp
Source: `plans/product-brief.md`, `.agents/references/research/grill-session.md`

A Terminal tab provides separate user terminal and agent-owned command terminal surfaces, with backend-owned lifecycle, deterministic command allowlist, approvals, sanitized environment, PTY lifecycle, and audit records.

## CAP-009: Extension Install And Connect

Status: accepted
Confidence: user-accepted
MVP: yes
Phase: mvp
Source: `plans/product-brief.md`, `.agents/references/research/grill-session.md`

Users can install or connect plugins, skills, and MCP servers. Each extension record shows provenance, granted scopes, workspace/project scope, data shared, runtime/tool access, enabled state, and action audit. Extensions can affect agent execution only after explicit per-extension enablement.
