# Overview

Git Integration covers detecting Git state and helping users inspect agent changes.

# Success Criteria

Requirement: The app displays Git status for the active project.
Expected Result: Current branch, dirty state, and changed files are visible.

Requirement: The user can inspect diffs.
Expected Result: File changes can be reviewed before the user leaves the session.

Requirement: Recovery support is review-first.
Expected Result: Git status, changed-file list, and diffs support manual recovery, but the app does not promise automatic rollback.

Requirement: Git inspection does not require approval.
Expected Result: Git status, branch display, changed-file list, and diffs are available without approval prompts and are logged when performed by the runtime.

# Functional Acceptance Criteria

Given a registered Git project
When the project opens
Then the current branch and dirty state are displayed.

Given the agent modifies a tracked file
When Git status refreshes
Then the changed file appears in the changed-file list.

Given a changed file exists
When the user opens the diff
Then the app displays the file-level diff.

Given the runtime performs Git inspection
When it reads branch, status, changed files, or diff
Then the inspection proceeds without approval and is visible in tool activity.

# Security Acceptance Criteria

Given the agent proposes a Git action that changes repository state
When approval is required
Then the action does not execute before user approval.

Given the agent proposes a destructive or remote-affecting Git action
When the approval prompt appears
Then the prompt marks the action as high risk.

# Performance Acceptance Criteria

Requirement: Git status for typical repositories under 1 GB loads within 10 seconds.
Expected Result: Project registration and review remain usable for validation projects.

# User Experience Acceptance Criteria

Given changed files exist
When the session is open
Then the changed-file list is visible from the main workflow.

Given an approved local action causes unwanted changes
When the user reviews Git state
Then the app shows changed files and diffs for manual recovery.

# Failure Conditions

 - Git state is not shown for a valid Git repository.
 - Changed files are missing after agent edits.
 - Diff view displays stale content after refresh.
 - Read-only Git inspection repeatedly asks for approval.
 - A Git state-changing action executes without approval.
 - The app claims automatic rollback for arbitrary local actions.

# Out Of Scope

 - Commit creation.
 - Pull request creation.
 - Branch management.
 - Worktree management, including creation, selection, inspection, dirty-state
   cleanup, failed cleanup recovery, branch reuse handling, submodule handling,
   Git LFS handling, and nested repository handling.
 - Merge conflict resolution.
 - Automatic rollback, snapshots, restore points, and undo stacks.
