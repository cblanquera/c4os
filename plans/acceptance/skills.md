# Overview

Skills are not part of the MVP execution surface. MVP acceptance verifies that skill functionality is not accidentally exposed or executed.

# Success Criteria

Requirement: Skills are not executable in MVP.
Expected Result: No skill can run scripts, load references, or alter agent behavior through the MVP UI.

Requirement: Root `AGENTS.md` remains the only MVP instruction convention displayed.
Expected Result: Skill catalogs are not shown as available MVP capabilities.

# Functional Acceptance Criteria

Given the MVP build
When the user opens settings or project context
Then no skill installation, discovery, or invocation workflow is available.

Given a project contains a `skills/` directory
When the project is registered
Then the app does not execute or auto-load those skills in MVP.

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

# Out Of Scope

 - Skill discovery.
 - Skill invocation.
 - Skill version conflict resolution.
 - Skill scripts, references, and assets.

