# MVP Requirements

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
Confidence: imported
MVP: unknown
Phase: mvp
Source: `plans/product-brief.md`
Related:
- CAP-008
- AC-008

Terminal sessions must be owned by backend process state, not renderer shell spawn.

## REQ-009: Interface Shell

Status: proposed
Confidence: imported
MVP: yes
Phase: mvp
Source: `plans/product-interface.md`
Related:
- CAP-001
- CAP-002
- AC-009

The app should use a three-panel desktop shell with left navigation, central session thread or empty-state prompt, and right tool tabs for Browser, Files, and Terminal.
