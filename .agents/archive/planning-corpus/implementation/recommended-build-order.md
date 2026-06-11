# Recommended Build Order

## Principle

Build the smallest vertical slice that proves the architecture can enforce the
MVP safety model before adding UI breadth.

## Order

### 1. Local Foundation

Build Tauri shell, backend command boundary, SQLite migrations, diagnostics,
and keychain abstraction.

Why first: every other feature depends on app-owned records and local backend
authority.

### 2. Provider Setup

Build OpenRouter credential setup, keychain references, selected model state,
standing disclosure, and active-session key update/revoke blocking.

Why second: runtime launch must use a stable credential reference and fixed
selected model.

### 3. Project Boundary

Build project registration, Git detection, project-root resolver,
secret-deny/symlink handling, root `AGENTS.md` display, and read-only file
browser.

Why third: approval and tool policy need a reliable project-root boundary.

### 4. Hardened Runtime Adapter Skeleton

Build OpenCode launch, event stream ingestion, event normalization,
instruction-source preflight, `/config` redaction, and stop mapping without
enabling state-changing tools yet.

Why fourth: validates the highest-risk integration path before feature breadth.

### 5. Approval Gateway

Build protected-action classification, pending prompt state, one-time allow,
deny, narrow shell session allow, answered approval ledger, and structured
denial mapping.

Why fifth: protected tools cannot be enabled before this exists.

### 6. File And Shell Tool Execution

Enable project-root file reads/writes, shell execution with filtered
environment, shell redaction/truncation/omission, and process supervision.

Why sixth: this completes the local-risk execution loop under policy control.

### 7. Session UI

Build transcript, tool activity timeline, runtime states, approval prompts,
stop, retry-as-new-action, and restart/crash recovery states.

Why seventh: UI should reflect already-enforced backend state, not become the
policy authority.

### 8. Review And Artifact Surfaces

Build changed-file list, Git diff viewer, tool-to-change context, and
text-like artifact records/viewer.

Why eighth: review surfaces become valuable after agent work can safely change
files.

### 9. MVP QA And Release Gates

Run code-level tests, macOS Apple Silicon app validation, end-to-end journeys,
and later Windows 11 x64 compatibility validation.

Why ninth: release gates require the implemented app.

## First Vertical Slice

The first meaningful slice should prove:

- app launches;
- provider key is stored by reference;
- project root is registered;
- OpenCode session starts through the hardened adapter;
- instruction sources are disclosed;
- assistant output streams into app-owned records;
- stop marks the run stopped and preserves records.

State-changing tools can remain disabled in this first slice.
