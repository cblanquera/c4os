# Product Context

Status: active
Created: 2026-06-18
Updated: 2026-06-18
Source Note: Imported from `plans/product-brief.md`.

## Summary

C4OS is a local-first desktop command center for AI-assisted work. It lets users open trusted project folders, run persistent agent sessions, manage providers and approvals, inspect artifacts, edit files, and use safe browser and terminal surfaces from an app-owned workspace.

## Target Users

- Technical power users who need stronger local project context than cloud chat and safer workflows than unconstrained scripts.
- Developers and small technical teams that use repositories, instruction files, skills, MCP, scripts, branches, and review artifacts.
- Knowledge workers with folder-based local projects who need persistent sessions, artifacts, provider control, and local memory.
- Future secondary users include team leads, administrators, plugin authors, MCP authors, and skill authors.

## Core Workflow

1. Open, create, clone, or select a local project folder.
2. Trust the folder before local file, Git, shell, instruction, skill, or approval-scoped workflows are available.
3. Start or resume a session from the project workspace.
4. Select provider and model settings.
5. Submit prompts through the composer.
6. Watch agent status, tool activity, approvals, artifacts, and runtime events in context.
7. Approve or deny sensitive actions.
8. Inspect files, artifacts, browser previews, and terminal output inside product safety boundaries.
9. Resume later with workspace, session, artifact, approval, provider, and memory state intact.

## Product Implications

- The product should feel like a workspace, not a prompt box.
- Underlying runtimes should remain implementation details behind an app-owned adapter.
- Trust, approvals, provider settings, artifacts, and persistence are product features, not runtime leftovers.
