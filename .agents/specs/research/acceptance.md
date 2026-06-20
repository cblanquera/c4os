# Research Acceptance Criteria

## AC-001: Trust Gate

Status: proposed
Confidence: imported
MVP: yes
Phase: mvp
Source: `plans/product-brief.md`

Given no trusted project folder exists, when the user opens the app, then the composer and local agent actions are unavailable until a folder is selected and trusted.

## AC-002: Descriptor Safety

Status: proposed
Confidence: imported
MVP: yes
Phase: mvp
Source: `plans/product-brief.md`

Given a `.c4os` workspace file is written, then it contains only non-secret references and does not include raw API keys, full transcripts, artifact archives, or private operational state by default.

## AC-003: Session Resume

Status: proposed
Confidence: imported
MVP: yes
Phase: mvp
Source: `plans/product-brief.md`

Given a user leaves and returns to a project session, then transcript, artifacts, selected provider/model, approval history, and run state references are available for continuation.

## AC-004: Key Storage

Status: proposed
Confidence: imported
MVP: yes
Phase: mvp
Source: `plans/product-brief.md`

Given a provider key is saved, then raw key material is stored only in secure storage and is not visible in workspace descriptors or normal app data.

## AC-005: Sensitive Action Approval

Status: proposed
Confidence: imported
MVP: yes
Phase: mvp
Source: `plans/product-brief.md`

Given a shell command, file mutation, credential use, MCP mutation, networked tool, worktree mutation, share/export, or destructive action is requested, then app-owned policy allows, asks, or denies before execution.

## AC-006: Trusted-Root File Access

Status: proposed
Confidence: imported
MVP: yes
Phase: mvp
Source: `plans/product-brief.md`

Given a file operation is requested, then traversal outside trusted roots and casual `.git` mutation are rejected before read or write behavior occurs.

## AC-007: Isolated HTML Preview

Status: proposed
Confidence: imported
MVP: yes
Phase: mvp
Source: `plans/product-brief.md`

Given generated HTML is previewed, then it cannot access provider credentials, arbitrary workspace files, shell state, or privileged app APIs.

## AC-008: Backend Terminal Boundary

Status: proposed
Confidence: imported
MVP: yes
Phase: mvp
Source: `plans/product-brief.md`

Given a terminal session starts, then process handles and lifecycle state are owned by the backend and renderer transport is bounded and auditable.

## AC-009: Three-Panel UI

Status: accepted
Confidence: evidence-backed
MVP: yes
Phase: mvp
Source: `plans/product-interface.md`, `wireframes/ui-handoff-spec.md`

Given the main app is open, then the layout has left navigation, central session content, and right tool tabs in Browser, Files, Terminal order.

## AC-010: App Start Trust Entry

Status: accepted
Confidence: evidence-backed
MVP: yes
Phase: mvp
Source: `wireframes/ui-handoff-spec.md`

Given no trusted project folder exists, when the app opens, then the first screen presents folder, clone, workspace-file, and recent-workspace entry points and does not expose prompt submission.

## AC-011: Composer Context Before Submission

Status: accepted
Confidence: evidence-backed
MVP: yes
Phase: mvp
Source: `wireframes/ui-handoff-spec.md`

Given the user can submit a prompt, then attachment context, approval policy, branch context, and provider/model context are visible before submission and active chat follow-up context cannot silently change branch or model.

## AC-012: Session Thread Event Surfaces

Status: accepted
Confidence: evidence-backed
MVP: yes
Phase: mvp
Source: `wireframes/ui-handoff-spec.md`

Given a session has user messages, agent messages, tool calls, run events, or approval waits, then the thread renders them as distinct structured surfaces with clear ownership, disclosure controls, and reachable follow-up composer.

## AC-013: Right Tool Surface Boundaries

Status: accepted
Confidence: evidence-backed
MVP: yes
Phase: mvp
Source: `wireframes/ui-handoff-spec.md`, `.agents/specs/research/poc/results.md`, `.agents/references/research/grill-session.md`

Given a workspace shell is open, when the user switches Browser, Files, or Terminal, then only the selected right-panel tool body is active and MVP behavior includes project-scoped Browser profile, trusted project file browsing/editing, user terminal, and agent-owned command terminal with approval/audit boundaries.

## AC-014: Settings Navigation Contract

Status: accepted
Confidence: evidence-backed
MVP: yes
Phase: mvp
Source: `wireframes/ui-handoff-spec.md`

Given the user opens Settings, then the settings shell exposes Providers, Models, Runtimes, Configuration, Plugins, Skills, and MCP Servers in order, with Back to app returning to the workspace shell.

## AC-015: Wireframe Placeholder Guard

Status: accepted
Confidence: evidence-backed
MVP: yes
Phase: mvp
Source: `wireframes/ui-handoff-spec.md`

Given implementation uses the r04 wireframe handoff, then sample project names, file names, model names, terminal output, URLs, copy, statuses, and grayscale styling are treated as illustrative unless separately promoted into product copy, configuration, or requirements.
