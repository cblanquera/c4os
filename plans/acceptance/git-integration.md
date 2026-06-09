# Overview

Git Integration covers detecting Git state and helping users inspect agent changes.

# Success Criteria

Requirement: The app displays Git status for the active project.
Expected Result: Current branch, dirty state, and changed files are visible.

Requirement: The user can inspect diffs.
Expected Result: File changes can be reviewed before the user leaves the session.

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

# Security Acceptance Criteria

Given the agent proposes a Git action that changes repository state
When approval is required
Then the action does not execute before user approval.

# Performance Acceptance Criteria

Requirement: Git status for typical repositories under 1 GB loads within 10 seconds.
Expected Result: Project registration and review remain usable for validation projects.

# User Experience Acceptance Criteria

Given changed files exist
When the session is open
Then the changed-file list is visible from the main workflow.

# Failure Conditions

 - Git state is not shown for a valid Git repository.
 - Changed files are missing after agent edits.
 - Diff view displays stale content after refresh.

# Out Of Scope

 - Commit creation.
 - Pull request creation.
 - Branch management.
 - Worktree management.
 - Merge conflict resolution.

