# Overview

OpenRouter Integration covers entering, storing, validating, and using OpenRouter credentials for MVP model-backed sessions.

# Success Criteria

Requirement: Users can configure OpenRouter credentials.
Expected Result: A valid configured key allows an agent session to start.

Requirement: Users understand off-device model calls.
Expected Result: Provider setup communicates that prompts and context are sent through OpenRouter.

Requirement: OpenRouter model traffic is not described as product telemetry.
Expected Result: The UI distinguishes required model-provider traffic from disabled product telemetry.

Requirement: Model metadata is informational.
Expected Result: Missing or stale OpenRouter pricing, capability, or route metadata is labeled and does not block provider-ready sessions.

Requirement: OpenRouter context is bounded.
Expected Result: Model calls include only active-session context and explicitly read allowed files, not hidden project-wide or secret content.

Requirement: OpenRouter context uses standing disclosure.
Expected Result: Provider setup discloses context data flow once, and model-call activity records source summaries without per-call approval prompts.

Requirement: Provider outages fail closed.
Expected Result: OpenRouter or network failures stop new model calls, preserve session state, and allow retry after recovery.

Requirement: Credential changes do not affect running sessions.
Expected Result: OpenRouter key update or revoke is blocked while a session is running.

# Functional Acceptance Criteria

Given a user enters an OpenRouter key
When the key is saved
Then the app reports provider configuration as ready.

Given provider configuration is ready
When the user starts a session
Then the runtime can use the configured provider path.

Given a session is running
When the user attempts to update or revoke the OpenRouter key
Then the app blocks the change and explains that the session must be stopped or complete first.

Given a session starts
When the runtime uses OpenRouter
Then the session uses the credential reference captured at session start until stopped or complete.

Given no session is running
When the user updates or revokes the OpenRouter key
Then the app may update provider readiness for future sessions.

Given a user starts a session with a selected OpenRouter model
When the runtime begins model calls
Then the effective provider path is OpenRouter and the effective model matches the selected session model.

Given provider authentication fails
When the user starts a session
Then the session does not start and an actionable provider error is shown.

Given OpenRouter or network access fails during a session
When the runtime attempts a new model call
Then the call fails with a clear provider or network error and no assistant continuation is synthesized.

Given a model call fails because of OpenRouter or network outage
When the user reviews the session
Then transcript, tool history, approvals, artifacts, and the failed-call state are preserved.

Given OpenRouter or network access recovers
When the user retries the model call
Then the app uses the same selected OpenRouter model unless the user starts a new session with a different model.

Given provider authentication succeeds
When OpenRouter pricing, capability, or route metadata is unavailable or stale
Then provider-ready state is not downgraded solely because metadata freshness is unknown.

Given the runtime makes an OpenRouter model call
When model context is assembled
Then it includes only the active session transcript, selected model/routing metadata, user-approved or policy-allowed tool results and summaries, and file contents explicitly read inside the selected project root.

Given a prior shell command has a persisted output summary
When model context includes that shell tool result
Then it includes only the bounded redacted/truncated summary, explicit output-omitted marker, and safe output summary reason labels from the tool record.

Given a prior shell command has a live terminal buffer or omitted raw output
When model context is assembled
Then live raw terminal text and omitted raw stdout/stderr are not included.

Given a prior shell command has redaction or truncation metadata
When model context is assembled
Then redacted substrings, sensitive raw byte counts, offsets, hashes, and reconstruction metadata are not included.

Given provider setup is complete
When a model call is about to be sent through OpenRouter
Then the app does not require a per-call model-context approval prompt.

Given a model call is recorded in session activity
When the user reviews the activity
Then the record includes a bounded context-source summary, such as transcript, explicit file reads, and tool summaries.

# Security Acceptance Criteria

Given an OpenRouter key is saved
When application metadata is inspected
Then the raw key is not present in SQLite records.

Given an OpenRouter key is saved
When the app persists credentials
Then the raw key is stored only in the OS keychain or platform credential store.

Given OS keychain or platform credential storage is unavailable
When the user attempts provider setup
Then setup fails with an actionable storage error before the key is used for sessions.

Given provider errors are logged
When logs are persisted
Then the raw key is not included.

Given the selected project contains files the runtime has not explicitly read
When an OpenRouter model call is made
Then those file contents are not included through whole-repo indexing or hidden background ingestion.

Given root `AGENTS.md` exists
When the app assembles model context
Then the app does not automatically inject that file into model context.

Given an artifact exists
When an OpenRouter model call is made
Then artifact contents are not included unless the artifact is referenced by the user or explicitly read under normal file-read rules.

Given a file matches secret-deny patterns
When an OpenRouter model call is made
Then the file contents are never included in model context.

# Performance Acceptance Criteria

Requirement: Provider readiness checks complete within 5 seconds when the provider is reachable.
Expected Result: Users receive timely setup feedback.

# User Experience Acceptance Criteria

Given provider setup succeeds
When the user returns to the session screen
Then the provider-ready state is visible.

Given product telemetry is disabled for MVP
When provider setup explains data flow
Then it still discloses that model prompts and context are sent through OpenRouter.

Given provider setup explains data flow
When the user configures OpenRouter
Then the disclosure explains that prompts and bounded context may leave the machine through OpenRouter.

# Failure Conditions

 - Invalid credentials are accepted as ready.
 - Raw provider key appears in logs, transcript, or database records.
 - Raw provider key is written to SQLite, project files, env files, shell environments, or custom app vault files.
 - Provider setup succeeds after keychain storage fails.
 - OpenRouter key update or revoke succeeds while a session is running.
 - A running session hot-swaps to a newly entered OpenRouter key.
 - A failed in-flight model call retries with a newly entered key.
 - Provider failure is shown as an unexplained runtime failure.
 - OpenRouter or network outage causes loss of transcript, tool history, approvals, or artifacts.
 - The app silently retries failed model calls in the background.
 - The app queues failed model calls for background resend.
 - The app switches to another model or provider after outage without a new user-started session.
 - The app shows a synthetic assistant continuation after provider failure.
 - The UI implies all data stays local while OpenRouter model calls are enabled.
 - Runtime model calls bypass OpenRouter or use a different model than the selected session model.
 - Whole-repo contents are sent to OpenRouter by default.
 - Hidden background file ingestion sends project files to OpenRouter.
 - The app automatically injects root `AGENTS.md` into model context.
 - Artifact contents are sent without user reference or explicit runtime read.
 - Secret-deny file contents are sent to OpenRouter.
 - The app requires per-call context approval before every model call.
 - Model-call activity omits context-source summaries.
 - MVP exposes raw prompt export, token-by-token context inspection, or editable context composition as required workflows.
 - Missing or stale model metadata is shown as current authoritative information.

# Out Of Scope

 - OpenRouter account management.
 - Cost dashboard.
 - App-owned live model catalog sync.
 - Budget enforcement.
 - BYOK provider subconfiguration inside OpenRouter.
 - Direct provider fallback.
 - Offline model fallback.
 - Automatic model switching after outage.
 - Queued background resend.
 - Synthetic assistant continuation after provider failure.
 - OpenRouter key export/import.
 - Hot key rotation.
 - Per-session credential switching.
 - Mid-call credential retry.
 - Custom encrypted secret vault.
 - Whole-repo indexing for model context.
 - Background context ingestion.
 - Per-call model-context approval.
 - Raw prompt export.
 - Token-by-token context inspection.
 - Editable context composer.
