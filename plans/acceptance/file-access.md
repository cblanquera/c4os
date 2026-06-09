# Overview

File Access covers MVP project-root scoped file reads and writes through approved runtime actions.

# Success Criteria

Requirement: File operations are scoped to the active project root.
Expected Result: No file write occurs outside the project root during MVP validation.

Requirement: File writes require approval.
Expected Result: File write actions do not execute before user approval.

# Functional Acceptance Criteria

Given an active project
When the agent reads a project file
Then the read target resolves inside the project root.

Given the agent proposes a file write
When the user has not approved it
Then the file write does not execute.

Given the user denies a file write
When the session continues
Then the denied write is recorded and the target file remains unchanged.

# Security Acceptance Criteria

Given a file path contains traversal segments or symlinks
When the app evaluates the path
Then the resolved path is checked against the project root boundary.

Given a file path matches configured secret-deny patterns
When the agent requests access
Then the action is blocked or requires the strictest available approval.

# Performance Acceptance Criteria

Requirement: File operation approval prompts appear before execution.
Expected Result: No measurable execution starts before the approval decision.

# User Experience Acceptance Criteria

Given a file action approval prompt
When it is displayed
Then the user sees the operation type and target path.

# Failure Conditions

 - A write occurs outside the project root.
 - A denied write modifies a file.
 - The approval prompt omits the target path.

# Out Of Scope

 - External-directory grants.
 - File operations outside project root.
 - Cloud-drive conflict handling.
 - Non-Git folder isolation.

