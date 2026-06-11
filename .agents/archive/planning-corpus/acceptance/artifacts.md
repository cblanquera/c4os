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

Given a generated file is neither text-like nor an accepted local raster image
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

 - Rich previews for PDFs, SVGs, non-raster images, spreadsheets, and
   documents.
 - Active HTML rendering.
 - Browser-based artifact rendering.
 - Chromium-backed artifact previews.
 - Artifact execution.
 - Artifact duplicate/export workflows.
 - Artifact search.
 - Artifact sharing.
 - Artifact marketplace extensions.
 - Automatic artifact ingestion into model context.

# Proposed V1 Rich Preview Tier

Status: accepted on 2026-06-12.

Accepted tier: `raster_image_preview_only`.

This tier would add passive previews for local raster image artifacts only:
PNG, JPEG, WebP, and GIF. It would not add SVG, HTML, PDF, document,
spreadsheet, browser, remote URL, execution, export, duplicate, search, or
automatic model-context behavior.

## Proposed Functional Acceptance Criteria

Given a local raster image artifact is recorded with project, session, and
source-tool provenance
When the user opens the artifact
Then the app displays a passive image preview and shows the artifact provenance.

Given a local raster image artifact is recorded
When the app stores or opens it
Then the app does not execute artifact content, run scripts, load remote URLs,
or grant privileged backend access to the preview surface.

Given a generated SVG, HTML, PDF, document, spreadsheet, archive, binary, or
remote URL artifact is recorded
When the user opens it
Then the app indicates that rich preview is unavailable and does not render it
as active or trusted content.

Given a raster image artifact exists
When model context is assembled for OpenRouter
Then the image bytes and derived image content are not included unless the user
explicitly asks the runtime to read or reference the file under normal
project-root file-read rules.

Given artifact records exist
When the user views the artifact list
Then artifact search, export, duplicate, sharing, marketplace, and browser
viewing controls remain unavailable.

## Proposed Security Acceptance Criteria

Given the preview tier is `raster_image_preview_only`
When an artifact preview is opened
Then the preview is local-only, passive, read-only, and unprivileged.

Given an artifact has an image-like extension but unsupported or suspicious
content
When the artifact is opened
Then preview fails closed with an unavailable reason instead of executing or
rendering active content.

Given active HTML or SVG content is present
When the artifact is opened
Then it is never rendered through the image preview path.

## Still Out Of Scope For Proposed Tier

- SVG preview.
- Active HTML rendering.
- PDF, document, spreadsheet, and archive previews.
- Browser or Chromium-backed artifact rendering.
- Remote URL previews.
- Artifact execution.
- Artifact search, export, duplicate, sharing, or marketplace workflows.
- OCR, DOM extraction, image analysis, screenshot understanding, or automatic
  model-context ingestion.
