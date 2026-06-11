# Objective

Understand performance and resource limits for large projects, active sessions, runtime processes, file watchers, and future concurrent agents.

# Context

Reviews identify scaling risks around multiple agents, MCP servers, shells, file watchers, SQLite writes, UI event streams, and large repositories.

# Questions To Answer

 - What repository size is acceptable for MVP validation?
 - How should monorepos, generated folders, vendor folders, binaries, nested repos, submodules, and Git LFS be handled?
 - How many active runtime sessions can be supported on a typical laptop?
 - What resource limits should be enforced for shell commands?
 - How is UI event backpressure handled?
 - What limits are needed before multi-agent support?

# Hypothesis

MVP should support one active session and typical repositories only, with explicit exclusions for very large or unusual workspaces until performance is measured.

# Investigation Plan

 - Define representative small, medium, and large workspace profiles.
 - Estimate file scan, Git status, and diff costs.
 - Review file watcher limits across platforms.
 - Model event volume from runtime tool activity.
 - Identify resource limits for shell and runtime processes.
 - Document MVP workspace size guidance.

# Success Criteria

 - MVP workspace size limits are recommended.
 - Resource constraints are documented.
 - Multi-agent scaling prerequisites are identified.
 - Large-workspace exclusions are explicit.

# Decisions Unlocked

 - ADR-006: Project, Session, And Worktree Model.
 - ADR-022: MVP Roadmap Sequencing.
 - Future multi-agent planning.

# Estimated Effort

2 to 4 engineering days.

