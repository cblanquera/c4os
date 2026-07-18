# Agent Workspace Terms

This file defines shared Agent Workspace terminology. Add project-specific terms outside the managed section.

<!-- agent-workspace-rules:start -->
## Managed Terms

- **Accepted Reusable Truth**: project knowledge approved or established enough to live in `.agents/context/`.
- **Agent Workspace**: everything inside a project root `.agents/` folder.
- **Agent Files**: markdown files under `.agents/` designed for AI-agent consumption unless explicitly excluded.
- **Context Demotion**: rerouting content out of `.agents/context/` when it is not Accepted Reusable Truth, is stale or contradicted, is too narrow for shared context, or belongs in a Spec File, Reference File, or Resource File. Demotion preserves useful material in the right lower-authority location; it is not deletion.
- **Context Files**: Agent Files under `.agents/context/`.
- **Context Promotion**: moving or copying accepted reusable truth from Source Material, Spec Files, Proofs, research findings, implementation evidence, or other Agent Files into `.agents/context/` so future agents can treat it as shared source-of-truth material.
- **Freeze/Frozen**: accepted planning state for a Spec File or spec package that should not be changed unless the user explicitly permits reopening it.
- **Gaps**: documented unknowns written as questions and based on the current Context Files plus the Agent Files in the relevant spec. Each Gap must be paired with an assumption, a decision, or an explicit unresolved status.
- **Intersection Scan**: review of Agent Files for overlaps, conflicts, answered gaps, new gaps, and affected records before updates.
- **Knowledge Base (KB)**: `.agents/context/`, the Agent Workspace source-of-truth folder containing Accepted Reusable Truth.
- **Proofs**: technical prototypes, experiments, or verification artifacts created to address Gaps before a spec is trusted for implementation.
- **Raw Source**: preserved verbatim or extracted source data stored under `.agents/resources/` before being rewritten into Agent Files.
- **Reference Files**: Agent Files under `.agents/references/`.
- **Reference Links**: markdown links from Agent Files to Reference Files with enough description for an agent to decide whether to load them.
- **Resource Files**: files under `.agents/resources/` that store Raw Source and other non-agent supporting material.
- **Resource Links**: markdown links from Agent Files to Resource Files.
- **Source Material**: import input such as files, URLs, screenshots with text, raw resources, pasted text, or ad hoc prompt text.
- **Source Provenance**: source path, URL, capture source, extraction limits, and access date when known.
- **Spec Files**: Agent Files found under `.agents/specs/*/`.
<!-- agent-workspace-rules:end -->

## Project Terms

- **C4OS**: the desktop AI workspace and assistant identity. C4OS organizes work around projects and chat sessions while its local Tauri/Rust core retains product-state and security authority.
- **C4OS Core**: the authoritative Tauri/Rust boundary that owns durable product records, native and secure-storage access, process supervision, updates, approvals, audit state, and the privileged tool gateway.
- **Execution Environment**: the Local, Docker, SSH, or other supervised target in which an authorized operation runs. It can perform an exact approved effect but does not own C4OS policy or audit authority.
- **Marketplace**: a catalog source from which C4OS can fetch validated plugin metadata for listing and search. Adding a marketplace does not install, enable, or execute its plugins.
- **Model-Capability Descriptor**: C4OS's versioned product contract for model identity, lifecycle, modalities, limits, reasoning, tool calling, structured output, streaming, generation controls, and caching or session requirements.
- **Effective Capabilities**: the narrow intersection of declared, adapter-normalized, and observed model capability with the execution environment, installed resources, active configuration, and C4OS policy.
- **Plugin**: a C4OS-managed bundle of metadata, skills, MCP or app declarations, settings schemas, and explicitly reviewed hooks. A plugin receives no native Rust or arbitrary shell authority merely by being installed.
- **Provenance**: the recorded provider, model, adapter and native runtime version, execution environment, configuration, capability, resource, and source-run information needed to explain a session, turn, run, approval, artifact, error, or audit event.
- **Response Artifact**: a structured File, Browser, or Terminal result attached to a chat response. It may remain inline or take focus in the center workspace while the conversation moves to a contextual sidebar pane.
- **Run Attempt**: one execution attempt for an immutable User Turn. A retry creates a new Run Attempt while preserving earlier output, tool effects, and audit records.
- **Runtime**: a user-selectable AI execution engine, initially OpenCode or Pi, supervised by C4OS through its own Runtime Adapter.
- **Runtime Adapter**: a C4OS-owned integration boundary that normalizes a runtime's identity, lifecycle, capabilities, configuration, events, and health without transferring C4OS policy authority to that runtime.
- **User Turn**: the immutable user prompt and attachment snapshot under which one or more Run Attempts may execute.
