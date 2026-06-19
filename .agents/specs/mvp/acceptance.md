# MVP Acceptance Criteria

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
MVP: unknown
Phase: mvp
Source: `plans/product-brief.md`

Given a terminal session starts, then process handles and lifecycle state are owned by the backend and renderer transport is bounded and auditable.

## AC-009: Three-Panel UI

Status: proposed
Confidence: imported
MVP: yes
Phase: mvp
Source: `plans/product-interface.md`

Given the main app is open, then the layout has left navigation, central session content, and right tool tabs in Browser, Files, Terminal order.
