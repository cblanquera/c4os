# Overview

Marketplace is excluded from MVP. Acceptance verifies that marketplace distribution and remote plugin acquisition are not exposed.

# Success Criteria

Requirement: No marketplace is available in MVP.
Expected Result: Users cannot browse, install, update, or enable marketplace plugins.

# Functional Acceptance Criteria

Given the MVP build
When the user opens navigation or settings
Then marketplace browsing is not available.

Given a marketplace manifest is present in a project
When the project is opened
Then the app does not fetch remote plugin sources.

# Security Acceptance Criteria

Given a marketplace entry references a remote plugin
When the app runs in MVP mode
Then no network request is made to install or update that plugin.

# Performance Acceptance Criteria

Requirement: Marketplace exclusion creates zero startup network dependency.
Expected Result: App startup succeeds without marketplace connectivity.

# User Experience Acceptance Criteria

Requirement: Users are not prompted to install marketplace content in MVP.
Expected Result: No marketplace install prompts appear during validation journeys.

# Failure Conditions

 - Marketplace content appears as installable.
 - Remote plugin sources are fetched.
 - Marketplace update checks run.

# Out Of Scope

 - Marketplace catalogs.
 - Plugin search and install.
 - Plugin updates.
 - Ratings, reviews, advisories, and curation.

