# Feature Goals

Status: planning input
Last Updated: 2026-06-18

This file captures high-level feature goals only. Before implementation, convert goals into bounded spec records, acceptance criteria, and progress items.

## Goals

- Folder-first workspace creation, opening, cloning, trust, and recent workspace access.
- Persistent project sessions with visible status, resumable transcript, selected provider/model, approval history, artifacts, and runtime references.
- Provider and model management for OpenAI-compatible providers with secure BYOK storage.
- App-owned approval gateway, saved rules, and audit logs.
- File explorer and editor inside trusted-root controls.
- Artifact storage and safe viewers for Markdown, text, code, diffs, HTML, images, PDFs, downloadable files, and command logs.
- User-owned Browser surface with project-scoped profile, local file browsing, public web browsing, and request-scoped agent browsing.
- Backend-owned terminal surface with trusted-root cwd validation, PTY lifecycle, sanitized environment, audit records, and bounded renderer transport.
- Extension install/connect flows for MCP servers, skills, and plugins, with explicit enablement before runtime impact.
- Local memory scoped to workspace, project, or session.
- Codex-like prework rendering, transcript polish, prompt composer controls, and messenger-style session layout.
