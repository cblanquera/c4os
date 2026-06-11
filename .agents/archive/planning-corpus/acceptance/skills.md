# Overview

Skills are not part of the MVP execution surface. MVP acceptance verifies that skill functionality is not accidentally exposed or executed.

# Success Criteria

Requirement: Skills are not executable in MVP.
Expected Result: No skill can run scripts, load references, or alter agent behavior through the MVP UI.

Requirement: Root `AGENTS.md` remains the only MVP instruction convention displayed.
Expected Result: Skill catalogs are not shown as available MVP capabilities.

Requirement: Root `AGENTS.md` is display-only app behavior.
Expected Result: The app does not automatically inject root `AGENTS.md` into app-owned model context or treat it as a permission source.

# Functional Acceptance Criteria

Given the MVP build
When the user opens settings or project context
Then no skill installation, discovery, or invocation workflow is available.

Given a project contains a `skills/` directory
When the project is registered
Then the app does not execute or auto-load those skills in MVP.

Given a project contains root `AGENTS.md`
When the project is registered
Then the app displays the file without using it to alter approvals, file access, shell access, Git access, or app-owned model context.

# Security Acceptance Criteria

Given a skill contains scripts or assets
When the project is opened
Then the app does not execute scripts or render skill assets as trusted content.

# Performance Acceptance Criteria

Requirement: Skill scanning does not run in MVP.
Expected Result: Project registration time is not affected by skill catalog size.

# User Experience Acceptance Criteria

Requirement: The MVP UI does not imply skills are supported.
Expected Result: Users cannot mistake unavailable skill features for broken behavior.

# Failure Conditions

 - A skill changes agent behavior in MVP.
 - Skill scripts execute.
 - Skill discovery appears as an enabled feature.
 - Root `AGENTS.md` changes permissions or approval behavior.
 - The app silently injects root `AGENTS.md` into app-owned model context.

# Out Of Scope

 - Skill discovery.
 - Skill invocation.
 - Skill version conflict resolution.
 - Skill scripts, references, and assets.

# V1 Acceptance Criteria

Status: accepted.

V1 may add `explicit_discovery_and_invocation_only` support.

## V1 Functional Acceptance Criteria

Given a selected local Git project contains project-local `SKILL.md` files
When the user opens the skills surface
Then the app may list skill name, description, source path, and support status.

Given a skill is listed
When the user selects it explicitly
Then the app may include the selected skill text in the current session only as
an explicit user-directed context addition or explicit project-root file read.

Given a skill exists in the project
When a session starts or a prompt is submitted
Then the app does not auto-invoke the skill, auto-load the skill, or silently
alter agent behavior.

## V1 Security Acceptance Criteria

Given a skill declares scripts, references, or assets
When the app discovers or displays the skill
Then the app does not execute scripts, run install steps, load references as
trusted context, render assets as trusted content, or grant additional
permissions.

Given a skill is selected
When the runtime requests file, shell, Git, or model-provider actions
Then the existing Approval Gateway and project-root rules still apply without
skill-based overrides.

Given a skill file is outside the selected project root or matches a
secret-deny path
When discovery or invocation is attempted
Then the app blocks the file under normal project-root and secret-deny rules.

## V1 User Experience Acceptance Criteria

Requirement: Users must not mistake V1 support for full Agent Skills
compatibility.
Expected Result: Skill UI labels unsupported capabilities such as scripts,
references, assets, auto-invocation, global skill catalogs, version conflict
resolution, plugin installation, and marketplace distribution as unavailable.

Requirement: Invocation is deliberate.
Expected Result: The user must explicitly choose a listed skill before its
content can be considered for the current session.

## V1 Failure Conditions

- A skill changes agent behavior without explicit user selection.
- A skill script executes.
- A skill changes permissions or approval behavior.
- A skill outside the selected project root is discovered or loaded.
- A skill file matching secret-deny rules is previewed or loaded.
- The app claims full Agent Skills compatibility.

## V1 Out Of Scope

- Auto-invocation.
- Global skill catalogs.
- User home skill discovery.
- Plugin-provided skills.
- Skill script execution.
- Skill dependency installation.
- Trusted rendering of skill assets.
- Automatic loading of skill references.
- Skill version conflict resolution.
- Skill marketplace workflows.
- Skill export, import, or round-trip compatibility.
