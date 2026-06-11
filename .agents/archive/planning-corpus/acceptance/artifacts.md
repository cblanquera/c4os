# Overview

Artifacts cover basic persisted outputs from MVP sessions: plain text, Markdown, logs, diffs, and generated source or config files.

# Success Criteria

Requirement: Basic artifacts are captured with provenance.
Expected Result: Each artifact is associated with a project, session, and source tool activity when applicable.

Requirement: MVP does not include artifact search.
Expected Result: Users can list and open visible artifact records, but cannot search across artifacts.

# Functional Acceptance Criteria

Given a tool creates a generated file
When the session records the result
Then the file appears as a changed project file and, when text-like, may appear as a basic artifact record.

Given a command produces log output
When the command completes
Then the output summary is available from the tool record.

Given an artifact is listed
When the user opens it
Then the app displays supported text-based content or indicates that preview is unavailable.

Given artifacts exist
When the user views the MVP artifact list
Then no artifact search UI is exposed.

Given a generated file is not text-like
When the user opens it from the artifact list
Then the app links to or reveals the file without rendering a rich preview.

# Security Acceptance Criteria

Given an artifact contains active HTML or executable content
When the user opens the artifact in MVP
Then the app does not execute that content as trusted code.

Given an artifact contains provider credentials
When persisted by the app
Then credentials are redacted or the artifact is not stored.

Given an artifact exists
When model context is assembled for OpenRouter
Then the artifact contents are not included unless the user references the artifact or the runtime explicitly reads the artifact under normal file-read rules.

# Performance Acceptance Criteria

Requirement: Opening MVP-supported text artifacts under 5 MB completes within 2 seconds.
Expected Result: Basic artifact review is responsive.

# User Experience Acceptance Criteria

Given an artifact has provenance
When the user views the artifact
Then the related session or tool activity is identifiable.

# Failure Conditions

 - Artifact provenance is missing.
 - Artifact search UI appears as an MVP feature.
 - Active content executes in an MVP artifact preview.
 - Browser or Chromium rendering is required to view an MVP artifact.
 - A rich media, PDF, document, or spreadsheet preview is treated as an MVP requirement.
 - Text artifact preview fails for supported file types.
 - Artifact contents are sent to OpenRouter without user reference or explicit runtime read.

# Out Of Scope

 - Rich previews for PDFs, images, spreadsheets, and documents.
 - Active HTML rendering.
 - Browser-based artifact rendering.
 - Chromium-backed artifact previews.
 - Artifact execution.
 - Artifact duplicate/export workflows.
 - Artifact search.
 - Artifact sharing.
 - Artifact marketplace extensions.
 - Automatic artifact ingestion into model context.
