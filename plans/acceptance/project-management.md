# Overview

Project Management covers registering and displaying one local Git project for MVP validation.

# Success Criteria

Requirement: A user can register one local Git project.
Expected Result: The app shows the selected project root, project name, current Git branch, and dirty state.

Requirement: The app communicates project scope.
Expected Result: The user can identify the folder that bounds file access before starting a session.

# Functional Acceptance Criteria

Given a user selects a local Git repository
When the project is added
Then the app displays the project root, repository status, current branch, and changed-file count.

Given a project has a root `AGENTS.md`
When the project is opened
Then the app displays that root instruction file as project guidance.

Given a user attempts to add a second active project during MVP validation
When the current MVP mode supports one active project
Then the app prevents ambiguity by keeping only one active project context visible.

# Security Acceptance Criteria

Given a project is registered
When any file operation is proposed
Then the operation is evaluated against the registered project root.

Given a path resolves outside the registered project root
When the agent proposes access to that path
Then the app blocks the operation unless the MVP policy explicitly supports external approval.

# Performance Acceptance Criteria

Requirement: Project registration for repositories under 1 GB completes within 10 seconds on validation machines.
Expected Result: Users can start setup without perceivable long indexing.

# User Experience Acceptance Criteria

Given a project is open
When the user views the project header
Then the root path and Git state are visible without opening settings.

# Failure Conditions

 - A non-Git folder is accepted as an MVP project.
 - The displayed project root differs from the access boundary.
 - Git state is missing or stale after project registration.

# Out Of Scope

 - Multiple active projects.
 - Non-Git project workflows.
 - Worktree management.
 - Cloud-synced workspace handling.

