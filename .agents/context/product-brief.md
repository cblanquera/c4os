# Product Brief

Status: active
Created: 2026-06-21
Updated: 2026-06-21
Source Note: Normalized from the previous context documents. Detailed source slices are preserved under `.agents/references/context/`.

## Purpose

Use this as the first context gate for C4OS product prework. It defines the product identity, user, goals, principles, and vocabulary, then routes agents to the smallest next document needed.

## Load When

- You need to understand what C4OS is, who it serves, or why it exists.
- You are starting product, planning, spec, review, handoff, or intake work.
- You need the document map before choosing a narrower context file.
- You need shared vocabulary before reading product, technical, creative, or work-order records.

## Skip When

- You already know the product frame and only need detailed feature behavior; load `product-specs.md`.
- You only need architecture, runtime, persistence, security, Browser, or Terminal constraints; load `technical-specs.md`.
- You only need UI layout, interaction, or visual direction; load `creative-specs.md`.
- You only need accepted sequencing, deferred work, or validation work packages; load `work-orders.md`.

## Owns

- Product thesis, target users, product goals, success measures, principles, and vocabulary routing.
- The five-document map for `.agents/context/`.

## Does Not Own

- Detailed feature inventory, implementation architecture, UI handoff detail, work sequencing, active progress, or traceable spec acceptance.

## Document Map

After this gate, load only the document that matches the task:

| Need | Load | Why |
| --- | --- | --- |
| Product identity, audience, goals, principles, vocabulary | Stay in `product-brief.md` | This file owns the product frame. |
| Customer workflow, feature surfaces, MVP product behavior | `product-specs.md` | Product specs own what the user can do. |
| Runtime, product model, persistence, trust, security, Browser, Terminal | `technical-specs.md` | Technical specs own enforcement and architecture. |
| UI layout, interaction behavior, visual tone, accessibility | `creative-specs.md` | Creative specs own the interface contract. |
| Accepted decisions, sequencing, deferred work, validation work packages | `work-orders.md` | Work orders own pre-execution routing. |

Expanded breakdowns live in `.agents/references/context/`. Source paths, provenance, imported chunks, and historical handoff material stay in references rather than in the five context entry documents.

## Reference Routing

| Need | Load | Why |
| --- | --- | --- |
| More complete product thesis, users, goals, workflow, principles | `.agents/references/context/product/product.md` | Expanded product context from the retired detailed document. |
| Exact vocabulary definitions | `.agents/references/context/product/terms.md` | Canonical term meanings and scope distinctions. |
| High-level feature goal inputs | `.agents/references/context/product/feature-goals.md` | Goal list preserved from prior context. |
| Source and artifact provenance | `.agents/references/context/source-provenance.md` | Source paths stay out of context docs. |

## Summary

C4OS is a folder-first local AI workspace for technical project work. It gives users a stronger interface than cloud chat and a safer interface than unconstrained local agent scripts by making trusted folders, sessions, approvals, artifacts, providers, runtime execution, Browser, Terminal, files, extensions, and local memory app-owned product concepts.

## Product Thesis

C4OS exists between cloud-first chat tools and developer-first agent runtimes. Cloud chat lacks safe ownership of local project context, files, approvals, credentials, and workspace state. Raw CLIs and runtimes expose too much implementation detail and fragment persistence, safety, artifacts, and multi-project workflows.

C4OS should feel like a durable desktop workspace for local projects, not a prompt box or marketing dashboard.

## Target Users

- Technical power users who need stronger local project context than cloud chat and safer workflows than unconstrained scripts.
- Developers and small technical teams that use repositories, instruction files, skills, MCP, scripts, branches, and review artifacts.
- Folder-based knowledge workers who need persistent sessions, artifacts, provider control, and local memory.
- Future secondary users include team leads, administrators, plugin authors, MCP authors, and skill authors.

## Product Goals

- Provide a folder-first desktop AI workspace for local projects.
- Keep project files, session state, approvals, artifacts, memory, and provider settings under app-owned local control.
- Support multiple trusted project folders and multiple resumable sessions per project.
- Allow agent runs to operate concurrently without mixing approvals, outputs, working directories, artifacts, or runtime events.
- Route sensitive actions through app-owned approval and audit boundaries.
- Support OpenAI-compatible provider profiles with bring-your-own-key storage.
- Respect useful ecosystem conventions such as `AGENTS.md`, `AGENTS.override.md`, `SKILL.md`, MCP, OpenCode configuration, and OpenCode plugins where practical.

## Success Measures

- A new user can open and trust a project folder without external docs.
- A user can start a session, choose a model, submit a prompt, approve an action, and inspect the result in one flow.
- A user can leave and return to a session with transcript, artifacts, selected model, and approval history intact.
- Sensitive file, shell, MCP, credential, Browser, extension, and destructive actions are blocked or approval-gated before execution.
- Raw provider keys never appear in workspace files or normal app data.
- Untrusted generated HTML cannot access privileged app APIs.
- File and terminal actions cannot escape trusted roots through traversal.
- Runtime lifecycle can start, stop, recover, and report errors reliably.
- Sessions can run concurrently without cross-run approval or output leakage.

## Principles

- Local-first by default.
- Project-first and folder-trusted.
- App-owned product model over raw runtime concepts.
- Explicit trust before execution.
- Standards-compatible, not proprietary-first.
- Safe surfaces over raw power.
- Durable work with lightweight workspace descriptor files.

## Vocabulary

The canonical vocabulary is preserved in `.agents/references/context/product/terms.md`. Core terms include Workspace, Workspace file, Project folder, Trust record, Session, Agent run, Approval request, Artifact, Provider profile, Model profile, Local memory record, Extension inventory item, Extension enablement, and Extension prompt tag.
