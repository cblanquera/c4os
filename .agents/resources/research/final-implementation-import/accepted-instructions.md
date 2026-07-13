Split goals into several specs

- 01 Shell Plugin Architecture Refactor
- 02 Core App Shell UX
- 03 Plugin System And Settings Management
- 04 Runtime And Tool Policy
- 05 Chat Prompt Interactions
- 06 File System Plugin
- 07 File Editor Plugin
- 08 Terminal Plugin
- 09 Chat Debug Plugin
- 10 Browser Plugin
- 11 Skills Settings

Each spec must include:

- `status.md`
- `requirements.md`
- `acceptance.md`
- `decisions.md`
- `risks.md`
- `evidence.md`
- `tasks.md`
- `traceability.md`
- maybe `poc/`

----

initial scoping for each spec. if unsure, just leave it as an open ended question, ill do a grill session later.

----

Topics to research online including Tauri & Tauri plugins, modularizing application shells, Codex plugin specifications, AI harness implementations including: 

- [Onelevenvy/flock](https://github.com/Onelevenvy/flock)
- [openchamber/openchamber](https://github.com/openchamber/openchamber)
- [thClaws/thClaws](https://github.com/thClaws/thClaws)
- [nomifun/nomifun-tauri](https://github.com/nomifun/nomifun-tauri) 

**Additional Topics To Research**

- Tauri permission generation and capability files for plugin-owned commands.
- Tauri multi-window/webview and panel mounting patterns.
- Tauri sidecars/process lifecycle for runtimes, terminals, and plugin backends.
- Tauri updater/signing/packaging if plugins may ship separately later.
- Cross-platform config paths using Tauri path APIs and Rust `directories`/`dirs`.
- Plugin supply-chain security: local plugin trust, malicious repo/plugin installs, signature/update policy.
- MCP Apps / MCP UI, because C4OS plugin panels may eventually overlap with tool-provided UI.
- Agent Skills spec beyond Codex docs, especially metadata and packaging interoperability.
- VS Code/workbench-style panel layout patterns: resizable side panels, activity bar, panel persistence.
- Browser/document preview safety: `.docx`, `.xlsx`, screenshots, annotations, sandbox boundaries.
- Attachment support by model/provider: images, files, structured references, and tool-readable attachments.

**Additional Harnesses Worth Researching**

- [Goose](https://block.github.io/goose/) / Block Goose: strong fit for extensions, MCP, tool permissions, and open agent standards. Wired also reports Goose was contributed into the Agentic AI Foundation alongside MCP and AGENTS.md.
- [OpenHands](https://github.com/All-Hands-AI/OpenHands): strong fit for sandboxed coding-agent runtime, browser/terminal/file tool orchestration, and benchmark harness design. Its paper describes code, CLI, browser, sandbox, multi-agent, and eval support.
- [Cline](https://github.com/cline/cline) or Roo Code: useful for IDE-style approvals, tool call UX, MCP integration, file edits, browser actions, and human-in-the-loop flows.
- [Continue](https://github.com/continuedev/continue): useful for provider/model config, context providers, IDE extension architecture, and tool integration.
- [Aider](https://github.com/Aider-AI/aider): useful for git-first chat, repository context, patch/edit workflow, and terminal-native ergonomics.
- [Open Interpreter](https://github.com/OpenInterpreter/open-interpreter): useful for computer-use style execution, local-first agent control, and permission boundaries.
- [OpenClaw](https://github.com/openclaw/openclaw): useful but high caution. It is relevant for autonomous local agents and messaging-first interfaces, but recent security/malware ecosystem reporting makes it more useful as a safety case than a design model.

Do the research, fine tune scope for each scope and document proofs that need to be done per spec before task implementation. Make a separate research documentation that includes general notes and findings.

----

grill session see `./grill-session/`

----

Workflow issues:

- ADRs currently in `docs/` this is incorrect. ADRs should be stored in their relative specs. If multiple specs use the same ADR, then copy ADR to each spec.
- Persist source of truth to be `.agents/context` (and references it links to)
- Specs should not rely on other specs for any truth about the overall project
