# Overview

V1 export/import support is limited to `project_json_export_only`.

# Support Tier

Status: accepted on 2026-06-12.

Accepted tier: `project_json_export_only`.

Import is deferred. V1 export produces a selected-project JSON manifest for
portable review and backup. The export is not a round-trip compatibility
format.

# Functional Acceptance Criteria

Given a selected project has sessions, messages, tool records, and artifact
records
When the user exports the project
Then the app creates a JSON export manifest containing project metadata,
sessions, messages, safe tool summaries, and artifact metadata.

Given export is available
When app status is requested
Then status reports `project_json_export_only`, export available, import
unavailable, and no round-trip compatibility claim.

Given artifacts exist
When the project is exported
Then the export includes artifact metadata and portable filenames, but does not
include artifact binary payloads or absolute local artifact paths.

# Security Acceptance Criteria

Given session messages, metadata, tool summaries, or artifact metadata contain
secret-shaped values
When the project is exported
Then secret-shaped lines or values are redacted from the export.

Given tool records exist
When the project is exported
Then bounded safe tool summaries may be included, but raw shell stdout/stderr,
live terminal buffers, omitted raw output, sensitive byte counts, offsets,
hashes, and reconstruction metadata are not exported.

Given provider credentials exist
When the project is exported
Then provider keys, keychain references, and credential values are not exported.

# Compatibility Acceptance Criteria

Given a project export exists
When compatibility information is shown
Then import, round-trip compatibility, OpenCode config compatibility, and
standards compatibility claims remain unavailable.

# Out Of Scope

- Import.
- Round-trip compatibility.
- Raw secrets or provider keys.
- Raw shell stdout/stderr.
- Absolute local paths.
- Artifact binary payload export.
- OpenCode config export/import.
- Cross-project export.
- Cloud sync, sharing, and marketplace workflows.
