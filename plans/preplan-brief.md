# AI Workspace Specification Project

We are designing a desktop AI workspace application.

The objective of this phase is NOT to implement the application.

The objective is to perform research, requirements gathering, standards discovery, architectural analysis, and produce complete implementation specifications before coding begins.

## Research First

Before producing any specifications, research and document existing conventions, standards, and patterns used by modern AI coding and agent applications.

At minimum investigate:

* SKILLS.md conventions
* AGENTS.md conventions
* MCP (Model Context Protocol)
* Plugin and Marketplace conventions used by Codex
* Project and workspace management conventions used by Codex
* Multi-project and worktree management approaches
* Permission and approval systems
* Tool execution models
* Local file access patterns
* Session and memory management patterns
* Artifact generation and artifact viewing patterns

Additionally identify any other relevant standards, conventions, or emerging patterns that should be considered.

Do not assume the above list is complete.

## Product Goals

The application should provide a general-purpose AI workspace.

The application is not limited to software engineering workflows.

The application should support coding, writing, research, analysis, operations, documentation, and other agent-assisted workflows.

The application should be extensible through commonly adopted standards rather than proprietary features whenever practical.

## Desired Capabilities

Research how existing products implement these capabilities and propose an architecture.

Capabilities include:

* Multiple projects
* Multiple sessions per project
* Multiple concurrent agents
* Local file access
* Local shell execution
* Git integration
* MCP integration
* Skills support
* Agent instructions support
* Plugin support
* Marketplace support
* Artifact generation
* Artifact viewing
* Browser/web content viewing
* Side panel navigation
* Project file browsing
* Session persistence
* Model switching
* BYOK model providers
* OpenRouter support
* Local-first workflows
* Approval workflows
* Security controls

## Architectural Constraints

Favor existing standards and conventions over custom solutions.

Avoid inventing proprietary formats when a widely adopted convention already exists.

Prefer interoperability with the broader AI tooling ecosystem.

The application should be designed so that users can share skills, plugins, MCP configurations, and workflows with other compatible AI tools whenever possible.

## Preferred Technical Direction

The current preferred architecture is:

AI GUI
  ↓
OpenCode Runtime
  ↓
OpenRouter (BYOK)

The GUI should be a desktop application, with Tauri preferred over Electron unless research identifies a strong reason to use Electron.

Reasons Tauri is currently preferred:

- smaller app footprint
- lower memory usage
- native system integration
- Rust backend compatibility
- better fit for local-first desktop tooling

However, the specification should still compare Tauri vs Electron and document tradeoffs before finalizing the choice.

The application should avoid building a custom agent runtime in the MVP if OpenCode Runtime can satisfy the core requirements.

## Deliverables

Create a folder called `plans` and produce the following documents:

1. Industry Standards Research
2. Requirements Specification
3. Functional Specification
4. Architecture Specification
5. Plugin System Specification
6. Marketplace Specification
7. Security Specification
8. Data Model Specification
9. UX Specification
10. Implementation Roadmap
11. Risk Analysis
12. Open Questions and Research Gaps

For every architectural recommendation:

* explain alternatives considered
* explain tradeoffs
* explain why the recommendation was selected

Do not begin implementation.

Focus entirely on producing high-quality specifications and supporting research.
