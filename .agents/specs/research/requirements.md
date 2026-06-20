# Research Requirements

## REQ-001: Trusted Folder Required

Status: proposed
Confidence: imported
MVP: yes
Phase: mvp
Source: `plans/product-brief.md`
Related:
- CAP-001
- AC-001

Local file, Git, shell, instruction, skill, and approval-scoped workflows must remain unavailable until a project folder is present and trusted.

## REQ-002: Workspace Descriptor Safety

Status: proposed
Confidence: imported
MVP: yes
Phase: mvp
Source: `plans/product-brief.md`
Related:
- CAP-001
- AC-002

`.c4os` workspace descriptor files may reference folders, display names, settings, provider/model preferences, and non-secret IDs, but must not contain raw secrets, full transcripts, artifact archives, or private operational state by default.

## REQ-003: Session Resume Data

Status: proposed
Confidence: imported
MVP: yes
Phase: mvp
Source: `plans/product-brief.md`
Related:
- CAP-002
- AC-003

Sessions must preserve enough transcript, message index, run status, selected agent, provider/model, approval, artifact, memory, runtime, and branch/worktree metadata to resume work later.

## REQ-004: Provider Secret Storage

Status: proposed
Confidence: imported
MVP: yes
Phase: mvp
Source: `plans/product-brief.md`
Related:
- CAP-003
- AC-004

Raw provider keys must be stored only in OS keychain or the selected secure storage abstraction.

## REQ-005: Approval Before Sensitive Execution

Status: proposed
Confidence: imported
MVP: yes
Phase: mvp
Source: `plans/product-brief.md`
Related:
- CAP-004
- AC-005

Shell commands, file writes/deletes, MCP mutation, credential use, networked tools, worktree mutation, share/export, and destructive actions must be evaluated by app-owned policy before execution.

## REQ-006: Trusted-Root File Editing

Status: proposed
Confidence: imported
MVP: yes
Phase: mvp
Source: `plans/product-brief.md`
Related:
- CAP-005
- AC-006

File loading, editing, saving, reverting, creating, renaming, and deleting must stay inside trusted roots and must not expose `.git` mutation casually.

## REQ-007: Isolated HTML Artifacts

Status: proposed
Confidence: imported
MVP: yes
Phase: mvp
Source: `plans/product-brief.md`
Related:
- CAP-006
- AC-007

Generated or untrusted HTML must render in an isolated preview surface without privileged app APIs.

## REQ-008: Backend-Owned Terminal

Status: proposed
Confidence: user-accepted
MVP: yes
Phase: mvp
Source: `plans/product-brief.md`, `.agents/references/research/grill-session.md`
Related:
- CAP-008
- AC-008

Terminal sessions must be owned by backend process state, not renderer shell spawn. MVP must distinguish user terminal input from the agent-owned command terminal, apply command approval policy, allow configured read-only commands by default, and audit agent command execution.

## REQ-015: Command Allowlist

Status: accepted
Confidence: user-accepted
MVP: yes
Phase: mvp
Source: `.agents/references/research/grill-session.md`
Related:
- CAP-008
- AC-008

C4OS must load an app-bundled default command allowlist that can be overridden or extended by workspace config. Read-only command classification must use exact allowlisted command patterns. The MVP default allowlist includes `pwd`, `ls`, `find`, `git status`, `git diff`, `git log`, and network read commands such as `curl`.

## REQ-009: Interface Shell

Status: accepted
Confidence: evidence-backed
MVP: yes
Phase: mvp
Source: `plans/product-interface.md`, `wireframes/ui-handoff-spec.md`
Related:
- CAP-001
- CAP-002
- AC-009

The app must use a three-panel desktop shell with left project/session navigation, central session thread or empty-state prompt, and right workspace tool tabs in Browser, Files, Terminal order.

## REQ-010: Trust-Scoped App Start

Status: accepted
Confidence: evidence-backed
MVP: yes
Phase: mvp
Source: `wireframes/ui-handoff-spec.md`
Related:
- CAP-001
- AC-001
- AC-010

The first screen must route users through trusted folder, repository clone, workspace file, or recent workspace selection before prompt submission or local agent action becomes available.

## REQ-011: Composer Context Controls

Status: accepted
Confidence: evidence-backed
MVP: yes
Phase: mvp
Source: `wireframes/ui-handoff-spec.md`
Related:
- CAP-002
- CAP-003
- CAP-004
- AC-011

The prompt composer must expose attachment, approval policy, branch context, and model context controls before submission, and active chat follow-up composers must show locked read-only branch and model context.

## REQ-012: Structured Session Thread

Status: accepted
Confidence: evidence-backed
MVP: yes
Phase: mvp
Source: `wireframes/ui-handoff-spec.md`
Related:
- CAP-002
- CAP-004
- AC-012

The session thread must distinguish user messages, agent messages, tool-call activity, run activity, and approval-waiting states as structured surfaces rather than flattening them into plain transcript text.

## REQ-013: Right Tool Surface Boundaries

Status: accepted
Confidence: evidence-backed
MVP: yes
Phase: mvp
Source: `wireframes/ui-handoff-spec.md`, `.agents/specs/research/poc/results.md`
Related:
- CAP-005
- CAP-006
- CAP-007
- CAP-008
- AC-013

The Browser, Files, and Terminal tabs must remain distinct right-panel surfaces. MVP behavior includes project-scoped Browser profile, local file browsing, public web browsing, request-scoped agent browsing, trusted project file explorer/editor states, user terminal, and agent-owned command terminal with approval/audit boundaries.

## REQ-014: Settings Surfaces

Status: accepted
Confidence: evidence-backed
MVP: yes
Phase: mvp
Source: `wireframes/ui-handoff-spec.md`
Related:
- CAP-003
- CAP-004
- CAP-009
- AC-014

Settings must expose Providers, Models, Runtimes, Configuration, Plugins, Skills, and MCP Servers in that order, while MVP execution impact remains limited by runtime, approval, provider, and explicit extension enablement decisions.

## REQ-016: Extension Install And Invocation

Status: accepted
Confidence: user-accepted
MVP: yes
Phase: mvp
Source: `.agents/references/research/grill-session.md`
Related:
- CAP-009
- CON-004
- DEC-005
- DEC-011

C4OS must support install/connect flows for plugins, skills, and MCP servers. Extension records must include source/provenance, granted scopes, workspace/project scope, data shared, runtime/tool access, enabled/disabled state, and audit log of extension actions. Runtime impact requires explicit per-extension enablement. Explicit chat invocation uses `$skill`, `@plugin`, and `^mcp` tags; MCP invocation must be explicit.
