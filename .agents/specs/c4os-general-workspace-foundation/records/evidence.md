# Evidence

## EVD-001: Current Project Selector State

Status: accepted
Confidence: evidence-backed
MVP: no
Source: `src-tauri/src/project_selector.rs`
Related:
- CAP-003

### Statement

The current selector lists registered projects, tracks exactly one active
project, and reports search, grouping, archive, delete, favorites, metadata
editing, cross-project views, non-Git projects, and worktree management as
unavailable.

## EVD-002: Current Session Catalog State

Status: accepted
Confidence: evidence-backed
MVP: no
Source: `src-tauri/src/session_catalog.rs`
Related:
- CAP-004

### Statement

The current session catalog lists project sessions newest first, identifies the
latest session, rejects cross-project session selection, blocks new runs during
active sessions, and reports search, archive, delete, and concurrent active
sessions as unavailable.

## EVD-003: Current UI Shell State

Status: accepted
Confidence: evidence-backed
MVP: no
Source: `src/main.js`
Related:
- CAP-001
- CAP-005

### Statement

The current UI renders a first-run workflow for provider, Git project path, and
prompt submission, plus status tiles for provider, project, instructions,
skills, MCP, tool ledger, session, approvals, recovery, changes, and artifacts.

## EVD-004: Current Backend Status Contract

Status: accepted
Confidence: evidence-backed
MVP: no
Source: `tests/backend-command-boundary.test.mjs`
Related:
- CON-002
- CON-003

### Statement

The backend status contract exposes current support tiers and deferred flags
for provider expansion, project selector controls, retention, memory,
compatibility, browser/web viewing, plugin/marketplace, artifacts, skills,
MCP, and portability.

## EVD-005: Parent Roadmap Audience Decision

Status: accepted
Confidence: evidence-backed
MVP: no
Source: `.agents/specs/c4os-general-workspace-roadmap/records/evidence.md`
Related:
- DEC-001

### Statement

The user selected research, writing, documentation, and analysis as the first
general-workspace audience priorities on 2026-06-12.

## EVD-006: User Accepted Foundation Scope Recommendations

Status: accepted
Confidence: evidence-backed
MVP: no
Source: user decision 2026-06-12
Related:
- Q-001
- Q-002
- Q-003
- Q-004
- DEC-004
- DEC-005
- DEC-006
- DEC-007

### Statement

The user accepted the readiness review recommendations to defer non-Git
folders, use lightweight workflow-purpose labels with no hidden context effect,
promote project search and workflow-purpose filtering first, and promote
session search/filtering plus workflow-purpose grouping first.

## EVD-007: Project And Session Tables Support Label Extension

Status: accepted
Confidence: evidence-backed
MVP: no
Source: `src-tauri/src/storage.rs`
Related:
- TASK-006
- DEC-008

### Statement

Current `projects` and `sessions` tables are the routing records for project
selector and session catalog state. They already support additive migrations,
such as session `pinned` and `archived` columns in migration 002.

## EVD-008: Workflow Labels Should Not Use Message Or Artifact Metadata

Status: accepted
Confidence: evidence-backed
MVP: no
Source: `src-tauri/src/storage.rs`, `src-tauri/src/export.rs`
Related:
- TASK-006
- DEC-008

### Statement

Messages are append-only transcript records and artifacts are output records
linked to projects, sessions, and tool calls. Workflow-purpose labels are
navigation metadata, so storing them in messages or artifacts would mix
workspace organization with transcript/output history.

## EVD-009: Export Can Carry Safe Workflow Labels Without Import Claim

Status: accepted
Confidence: evidence-backed
MVP: no
Source: `src-tauri/src/export.rs`, `tests/backend-command-boundary.test.mjs`
Related:
- TASK-006
- DEC-009

### Statement

Project JSON export currently includes project and session metadata while
excluding absolute local paths, provider keys, raw secrets, raw shell output,
artifact payloads, and import/round-trip claims. Workflow-purpose labels can be
included as safe bounded metadata while import remains unavailable.

## EVD-010: Archived Session Delete Is Compatible With Session Labels

Status: accepted
Confidence: evidence-backed
MVP: no
Source: `src-tauri/src/storage.rs`
Related:
- TASK-006
- DEC-008

### Statement

Archived-session delete removes approvals, artifacts, adapter refs, tool calls,
messages, and the session row after checking archived, pinned, active, and
latest-session guards. A session-level workflow label stored on the session row
would be removed with that row and does not require separate cleanup.

## EVD-011: Status Contract Needs Explicit Workflow Label Capability

Status: accepted
Confidence: evidence-backed
MVP: no
Source: `src/backend/commands.js`, `tests/backend-command-boundary.test.mjs`
Related:
- TASK-006
- TASK-005

### Statement

The backend status contract currently exposes project selector, session,
retention, memory, compatibility, browser/web, plugin/marketplace, skills,
MCP, artifacts, and portability capability flags. Workflow-purpose labels need
an explicit status surface so UI copy does not imply hidden context, memory, or
compatibility behavior.

## EVD-012: Workflow Acceptance Examples Defined

Status: accepted
Confidence: evidence-backed
MVP: no
Source: item-051
Related:
- AC-007
- TASK-001

### Statement

Concrete first-slice examples now exist for research project resume, writing
draft iteration, documentation review, and analysis session comparison using
workflow-purpose labels and existing project/session/artifact surfaces.

## EVD-013: Workspace Home Shape Defined

Status: accepted
Confidence: evidence-backed
MVP: no
Source: item-051
Related:
- DEC-011
- TASK-001

### Statement

The first workspace home shape is a work-focused overview with project
search/filtering, workflow-purpose filtering, latest/resumable session state,
and compact capability status.

## EVD-014: Status Copy Boundary Defined

Status: accepted
Confidence: evidence-backed
MVP: no
Source: item-051
Related:
- DEC-012
- TASK-005

### Statement

Status copy is bounded to capability-level availability and deferral labels. It
does not claim broad compatibility, browser support, durable memory,
import/round-trip support, or hidden context behavior.
