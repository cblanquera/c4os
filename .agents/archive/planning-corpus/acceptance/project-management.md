# Overview

Project Management covers registering local Git projects and using a minimal Project Selector to choose one selected project for MVP validation.

# Success Criteria

Requirement: A user can register local Git projects.
Expected Result: The app shows the selected project root, project name, current Git branch, and dirty state.

Requirement: A user can choose one selected project from registered projects.
Expected Result: The app keeps one project context visible and active, without exposing full project-management workflows.

Requirement: The app communicates project scope.
Expected Result: The user can identify the folder that bounds file access before starting a session.

# Functional Acceptance Criteria

Given a user selects a local Git repository
When the project is added
Then the app displays the project root, repository status, current branch, and changed-file count.

Given a project has a root `AGENTS.md`
When the project is opened
Then the app displays that root instruction file as project guidance.

Given a project has a root `AGENTS.md`
When the app displays it
Then the app does not treat it as a permission source or automatically inject it into app-owned model context.

Given a project has a root `AGENTS.md`
When the user explicitly asks the agent to read it
Then the app treats the read as a normal logged project-root file read, not as hidden policy loading.

Given a project has a root `AGENTS.md`
When the user approves an agent-proposed edit to it
Then the app treats the edit as a normal project-root file write, not as instruction or permission reconfiguration.

Given a project has nested `AGENTS.md` files
When instruction preflight runs
Then the app discloses an ordered guidance stack with root guidance first and deeper nested guidance after its parent path.

Given nested `AGENTS.md` files contain overlapping guidance
When the app displays instruction resolution
Then conflict diagnostics are limited to ordered source disclosure and do not semantically merge, rewrite, or enforce guidance.

Given a nested `AGENTS.md` file is disclosed
When permissions or approvals are evaluated
Then the nested instruction file has no permission effect.

Given nested `AGENTS.md` files are disclosed
When OpenRouter-bound app-owned model context is assembled
Then the files are not included unless explicitly read under normal project-root file-read rules or runtime-native instruction loading is separately disclosed.

Given OpenCode has runtime-native instruction loading behavior
When the runtime control spike evaluates it
Then the app can observe and disclose the loaded instruction files, or disable that behavior.

Given OpenCode invisibly loads root or nested instruction files
When the app cannot observe, disclose, or disable that behavior
Then OpenCode is not accepted as the direct MVP runtime.

Given a user attempts to add a second active project during MVP validation
When the current MVP mode supports one active project
Then the app prevents ambiguity by keeping only one active project context visible.

Given multiple projects are registered
When the user opens the Project Selector
Then the app lists registered projects and allows selecting exactly one active project.

# Security Acceptance Criteria

Given a project is registered
When any file operation is proposed
Then the operation is evaluated against the registered project root.

Given a path resolves outside the registered project root
When the agent proposes access to that path
Then the app blocks the operation.

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
 - Root `AGENTS.md` changes permissions or approval behavior.
 - Editing root `AGENTS.md` automatically reloads model context, permission state, or instruction precedence.
 - Nested `AGENTS.md` changes permissions, approvals, or app-owned model context without explicit read or runtime-native disclosure.
 - OpenCode invisibly injects instruction files into model context without app visibility or disclosure.
 - The Project Selector exposes project search, grouping, archive, delete, favorites, metadata editing, cross-project views, or multi-project operations in MVP.

# Out Of Scope

 - Multiple active projects.
 - Full project management.
 - Project search, grouping, archive, delete, favorites, and metadata editing.
 - Cross-project views and multi-project operations.
 - Non-Git project workflows.
 - Worktree management.
 - Full AGENTS.md compatibility, semantic conflict resolution, export, or standards round-trip behavior.
 - Cloud-synced workspace handling.
