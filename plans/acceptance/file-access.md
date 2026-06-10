# Overview

File Access covers MVP project-root scoped file reads and approval-gated file writes.

# Success Criteria

Requirement: File operations are scoped to the active project root.
Expected Result: No file read or write occurs outside the project root during MVP validation.

Requirement: Project-root reads do not require per-read approval.
Expected Result: Runtime reads inside the project root proceed without prompting and are visible in tool activity.

Requirement: File writes require approval.
Expected Result: File write actions do not execute before user approval.

Requirement: MVP file browser is read-only.
Expected Result: Users can inspect files inside the selected project root, but cannot edit files directly through the file browser.

Requirement: MVP file browser has no project-wide content search UI.
Expected Result: Users can browse/open files for inspection, while runtime glob/grep tools remain available inside the selected project under normal file-access rules.

Requirement: Secret-deny files are blocked.
Expected Result: Agent reads and writes for likely secret files are denied even when the files are inside the selected project root.

# Functional Acceptance Criteria

Given an active project
When the agent reads a project file
Then the read target resolves inside the project root and the read is recorded in tool activity.

Given a project has a root `AGENTS.md`
When the user or runtime explicitly reads it during a session
Then the read follows normal project-root file-read rules and is recorded in tool activity.

Given a user opens a file in the MVP file browser
When the file resolves inside the selected project root
Then the app displays it read-only for inspection.

Given a user opens a file in the MVP file browser
When the file would resolve outside the selected project root
Then the app blocks the file open.

Given the agent requests a file read outside the project root
When the app resolves the path
Then the read is blocked under MVP rules.

Given the user opens the MVP file browser
When they inspect project files
Then no project-wide file content search UI is exposed.

Given the runtime uses glob or grep during an agent run
When it searches project files
Then the search remains scoped to the selected project root and respects secret-deny rules.

Given the agent proposes a file write
When the user has not approved it
Then the file write does not execute.

Given a project has a root `AGENTS.md`
When the agent proposes editing it
Then the edit follows normal project-root file-write approval rules.

Given a root `AGENTS.md` edit is approved and applied
When the session continues
Then the app does not automatically reload it into model context, permission state, or instruction precedence rules.

Given the user denies a file write
When the session continues
Then the denied write is recorded and the target file remains unchanged.

# Security Acceptance Criteria

Given a file path contains traversal segments or symlinks
When the app evaluates the path
Then the resolved path is checked against the project root boundary.

Given a symlink exists inside the selected project root
When the symlink resolves to a target inside the selected project root
Then file access may proceed under the normal read or write approval rules.

Given a symlink exists inside the selected project root
When the symlink resolves to a target outside the selected project root
Then the app blocks reads and writes through that symlink.

Given a file path matches built-in secret-deny patterns
When the agent requests access
Then the action is blocked with no MVP approval override.

Given a file path matches built-in secret-deny patterns
When the user opens the file in the MVP file browser
Then the app may show that the file exists but does not preview the file contents.

Given a file has not been explicitly read by the runtime
When model context is assembled for OpenRouter
Then the file contents are not included through background ingestion or whole-repo indexing.

Given root `AGENTS.md` has only been displayed by the app
When model context is assembled for OpenRouter
Then its contents are not included unless the file was explicitly read under normal project-root file-read rules.

Given a file path matches built-in secret-deny patterns
When model context is assembled for OpenRouter
Then the file contents are never included.

# Performance Acceptance Criteria

Requirement: File operation approval prompts appear before execution.
Expected Result: No measurable execution starts before the approval decision.

# User Experience Acceptance Criteria

Given a file action approval prompt
When it is displayed
Then the user sees the operation type and target path.

Given a file write approval prompt
When a bounded diff or summary can be produced safely
Then the prompt shows the target path, action type, and bounded diff or summary before approval.

Given a file write approval prompt
When a safe diff cannot be produced
Then the prompt clearly states that limitation and still requires explicit approval before execution.

Given the agent proposes editing root `AGENTS.md`
When the file write approval prompt is displayed
Then the prompt follows the same target path, action type, and bounded diff or summary rules as other project files.

Given the agent proposes multiple file writes
When the approval prompt is displayed
Then the user sees every target path, action type, and per-file bounded diff or summary state before approval.

Given the agent proposes more than 10 file writes in one batch
When the app evaluates the batch
Then the whole batch is blocked and the agent must propose smaller batches.

Given a batch file write proposal exceeds the bounded total preview size
When the app evaluates the batch
Then the whole batch is blocked and the agent must propose smaller batches.

Given the user opens MVP settings
When approval behavior is displayed
Then batch file caps and preview-size caps are not user-configurable.

Given a batch file write proposal contains an outside-root, secret-deny, or no-safe-preview item
When the approval prompt is displayed
Then that item is clearly identified before any approval decision.

Given a batch file write proposal contains an outside-root or secret-deny item
When the app evaluates the batch
Then the whole batch is blocked and no file writes execute.

Given a batch file write proposal contains a no-safe-preview item
When the approval prompt is displayed
Then the batch remains approve-or-deny as proposed without checkbox-per-file partial approval.

Given a batch file write approval is denied or blocked by policy
When the session continues
Then no hidden partial file writes execute.

Given a file write approval prompt is pending
When the app closes, crashes, is force-quit, or restarts
Then the prompt is discarded and the write cannot be approved until the agent proposes it again.

# Failure Conditions

 - A write occurs outside the project root.
 - A read occurs outside the project root.
 - A denied write modifies a file.
 - The file browser edits a file directly.
 - The file browser opens files outside the selected project root.
 - The file browser exposes project-wide file content search UI.
 - Project-root reads require repeated approval prompts.
 - Runtime file reads are not visible in tool activity.
 - The approval prompt omits the target path.
 - A pending file write approval survives app close, crash, force-quit, or restart as an approvable prompt.
 - A stale file write approval executes after restart.
 - A file write approval prompt omits the action type.
 - A file write approval prompt omits a safely available bounded diff or summary.
 - A file write approval prompt cannot produce a safe diff but fails to say so clearly.
 - A batch file write approval hides any target path, action type, or per-file preview state.
 - A batch file write approval hides outside-root, secret-deny, or no-safe-preview items.
 - A batch file write approval contains more than 10 files.
 - A batch file write approval exceeds bounded total preview size.
 - An oversized batch is approved by summary only.
 - The user can configure batch file caps, preview-size caps, or per-project approval thresholds in MVP.
 - A blocked batch writes any file.
 - A denied batch writes any file.
 - The app offers checkbox-per-file partial approval or automatically executes only the allowed subset.
 - The app rewrites a proposed batch into a smaller batch without the agent proposing it.
 - The UI offers blanket approve-all-future-writes or project-wide write approval.
 - The UI shows a giant scrollback approval prompt or unbounded generated diff.
 - A symlink escape allows reading or writing outside the selected project root.
 - An agent reads or writes a secret-deny file.
 - The file browser previews secret-deny file contents.
 - Model context includes file contents that were not explicitly read by the runtime.
 - Model context includes secret-deny file contents.

# Out Of Scope

 - External-directory grants.
 - Full editor or merge UI inside approval prompts.
 - Multi-file review workflow beyond the approval prompt, changed-file list, and diff viewer.
 - File operations outside project root.
 - Project-wide file content search UI.
 - Whole-repo indexing for model context.
 - Hidden background file ingestion.
 - Built-in file editor.
 - External-editor workflow.
 - Cloud-drive conflict handling.
 - Non-Git folder isolation.
