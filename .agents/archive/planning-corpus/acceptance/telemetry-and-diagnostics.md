# Overview

Telemetry and Diagnostics covers the MVP privacy baseline for product telemetry, local diagnostics, and off-device model traffic.

# Success Criteria

Requirement: MVP sends no product telemetry.
Expected Result: The app does not send usage analytics, automatic crash reports, diagnostics, transcripts, tool outputs, artifacts, or support bundles to the product operator.

Requirement: Diagnostics are local-only.
Expected Result: Troubleshooting logs remain on device unless a future non-MVP export flow is explicitly added.

Requirement: OpenRouter model traffic is disclosed separately.
Expected Result: Users understand that model prompts and context leave the machine through OpenRouter even though product telemetry is disabled.

# Functional Acceptance Criteria

Given the MVP app is running
When normal product actions occur
Then no product telemetry request is sent.

Given a runtime, provider, shell, or tool failure occurs
When diagnostics are recorded
Then diagnostic records are stored locally only.

Given a model-backed session starts
When provider setup or session UI explains data flow
Then it states that prompts and context are sent through OpenRouter.

# Security Acceptance Criteria

Given local diagnostics are persisted
When logs are inspected
Then provider credentials and known secret values are redacted or absent.

Given a crash or runtime failure occurs
When the app recovers
Then no automatic crash upload or support bundle upload occurs.

Given approval records exist in the local activity ledger
When MVP diagnostics or troubleshooting UI is used
Then there is no approval-record export, copy-all, JSON download, support bundle, or share workflow.

Given approval records exist in the local activity ledger
When the user selects visible text manually
Then normal OS text copy is not treated as an approval-record export workflow.

Given shell output summaries exist in local diagnostics or the activity timeline
When the user selects visible redacted/truncated summary text manually
Then normal OS text copy is not treated as a raw shell output export workflow.

# Failure Conditions

 - Product usage analytics are sent in MVP.
 - Crash reports upload automatically.
 - Support bundles upload automatically.
 - Approval records can be exported, copied in bulk, downloaded as JSON, shared, or included in a support bundle.
 - Raw shell stdout/stderr can be persisted, copied through a dedicated raw-output button, exported, shared, or included in a support bundle.
 - Local diagnostics contain raw provider credentials.
 - The UI implies all data stays local while OpenRouter model calls are enabled.

# Out Of Scope

 - Product analytics.
 - Automatic crash reporting.
 - Remote diagnostics upload.
 - Support bundle upload.
 - Approval record export or sharing.
 - Enterprise telemetry policy.
