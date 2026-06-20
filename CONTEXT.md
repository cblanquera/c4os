# C4OS

C4OS is a desktop environment for agent-assisted work across coding, writing, research, analysis, operations, and documentation. The product language should describe user-facing workspace concepts rather than internal runtime implementation details.

## Language

**Workspace**:
A saved working environment that groups one or more project folders, sessions, settings, and local state. In MVP, a workspace must have at least one project folder before prompting is available.
_Avoid_: OpenCode project, repo wrapper

**Workspace File**:
A portable descriptor file that references one or more project folders and workspace-level settings without embedding secrets, transcripts, artifacts, or complete local history.
_Avoid_: Workspace archive, session bundle

**Local Operational State**:
Private app state stored on the local machine, including session indexes, approval audit records, artifact indexes, and runtime references.
_Avoid_: Workspace file contents, shared workspace state

**Project Folder**:
A local filesystem folder included in a workspace.
_Avoid_: OpenCode project, root

**Worktree**:
A Git working tree used to isolate a coding session from another branch or session.
_Avoid_: Workspace, project folder

**Session**:
A persistent conversation and work context inside a workspace.
_Avoid_: OpenCode session when speaking to users

**Memory**:
App-owned contextual knowledge carried across sessions within an explicit workspace, project, or session scope.
_Avoid_: Provider memory, runtime state

**Pinned Fact**:
A user-approved memory item that should remain visible and reusable until the user removes it.
_Avoid_: Transcript quote, hidden memory

**Session Summary**:
A compact app-owned summary of a session used for review, search, and future context assembly.
_Avoid_: Full transcript, provider memory

**Agent Run**:
An active or completed execution of an assistant against a session.
_Avoid_: Runtime job, OpenCode run

**Approval**:
A user decision that allows, denies, or remembers a sensitive action requested by an agent.
_Avoid_: OpenCode permission when speaking to users

**Artifact**:
A generated file or previewable output associated with a session.
_Avoid_: Output blob, runtime file

**Preview Surface**:
An unprivileged viewing area for artifacts or web content that must not expose privileged app APIs to untrusted generated content.
_Avoid_: Main renderer, app bridge view

**Provider**:
A model access route or service, such as OpenRouter, shown to users as the source of model capability.
_Avoid_: Runtime provider config

**Runtime Adapter**:
The internal boundary between the desktop app and the agent runtime implementation.
_Avoid_: OpenCode GUI, runtime UI
