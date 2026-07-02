# File System Plugin Decisions

Status: proposed

### DEC-001: 006: C4OS Grill Question 006 - User Config Location

Source: `.agents/references/research/final-implementation-import/grill-session/006-c4os-grill-question-006-user-config-location.json`

  - Where should C4OS user-global config live?: Other
  - For Windows, what should the equivalent be?: Other
  - Notes: You can decide this. Follow industry standard for app config per user location depending on OS. Somewhat related: In the current MVP it puts project level config in `.c4os` folder in the project root, I didnt want to pollute project folders with an app hidden folder. I wanted it all in a config per user folder instead.

### DEC-002: 007: C4OS Grill Question 007 - Workspace Descriptor Boundary

Source: `.agents/references/research/final-implementation-import/grill-session/007-c4os-grill-question-007-workspace-descriptor-boundary.json`

  - Should C4OS still have portable workspace descriptor files?: Yes, explicit portable workspace files only; no automatic project-root folders
  - If descriptor files exist, what can they contain?: Project folder references and shareable workspace settings only
  - Notes: You still should be able to load/save workspace files like in the mvp `workspace.c4os.json`. The main issue was how the MVP assigns project IDs (since project folder names are not unique). It relied on every project to have a `.c4os` folder so it can tag the project ID to that folder. I'm thinking a better way is to just create a map (id to folder) in a global config (or app level config). But if you have an even better idea let me know.

### DEC-003: 008: C4OS Grill Question 008 - Project Identity

Source: `.agents/references/research/final-implementation-import/grill-session/008-c4os-grill-question-008-project-identity.json`

  - How should C4OS assign stable project identity without project-root .c4os folders?: Canonical folder path is the project ID
  - What happens when a registered project folder moves or disappears?: Keep record as missing and allow relink/remove/reveal last known path
  - Notes: The frontend should mute (gray color) that project with a strike through. Then in the ... popover have a relocate option. Chat sessions under that project become read only until a location is found.

### DEC-004: 008A: C4OS Grill Question 008A - Project Relocation Identity

Source: `.agents/references/research/final-implementation-import/grill-session/008a-c4os-grill-question-008a-project-relocation-identity.json`

  - If canonical folder path is the project ID, what happens after relocation?: Relocated canonical path replaces the project ID and sessions are migrated
  - How should missing project sessions behave?: Muted project row with strike-through; sessions visible read-only until relocated

### DEC-005: 009: C4OS Grill Question 009 - Workspace Registry Layout

Source: `.agents/references/research/final-implementation-import/grill-session/009-c4os-grill-question-009-workspace-registry-layout.json`

  - How should user-config-owned workspaces and projects be stored?: One file per workspace with no central registry index
  - How should saved workspace.c4os.json files relate to the user registry?: Other
  - Notes: workspace is purely a list of project folders and the workspace name. chat sessions (in all cases) are stored on the user level. When a workspace file is loaded it looks for its relative chat session on the user level storage. This makes it possible for many workspace files to have the same project folders, while retaining it's relative chats when loaded.

### DEC-006: 009A: C4OS Grill Question 009A - Workspace Chat Identity

Source: `.agents/references/research/final-implementation-import/grill-session/009a-c4os-grill-question-009a-workspace-chat-identity.json`

  - How should user-level chat sessions be associated with a loaded workspace file?: Each project folder path owns its chats regardless of workspace file
  - If two workspace files contain the same project folders, should they share chats?: Yes, same project folders share chats across workspace files
  - Notes: A case study is I have a workspace for just my open source projects and another workspace for my work related projects. But some of my work related projects use some of my open source projects. In the case where I need to fix my open source project accepted in order to make my work project work they would live in the same workspace rather than me toggling between 2 workspaces. At the same time when I open my open source workspace I would like to see the latest changes.

### DEC-007: 010: C4OS Grill Question 010 - Unassigned Chat Scope

Source: `.agents/references/research/final-implementation-import/grill-session/010-c4os-grill-question-010-unassigned-chat-scope.json`

  - Where should unassigned chats live?: User-global shell chat history
  - What should the unassigned chat area be called in the UI?: Chats

### DEC-008: 021A: C4OS Grill Question 021A - Read Approval Boundary

Source: `.agents/references/research/final-implementation-import/grill-session/021a-c4os-grill-question-021a-read-approval-boundary.json`

  - When you say allow all reads and previews, what boundary applies?: Allow all filesystem reads without asking
  - Should previews follow the same boundary as reads?: Yes, previews follow the same boundary as reads
  - Notes: This will be the 3rd time I mentioned this, so i hope it's recorded this time. This app will be installed on a personal computer for personal consumption. It should be assumed that if a user wants to read any file on their computer, they should be allowed to. same goes if a user asks an agent to read a file. If an agent tries to read a file outside of the project folder and the user did not explicitly ask for it to be read, then the app should ask for permissions.

### DEC-009: 022: C4OS Grill Question 022 - Mutating Tool Defaults

Source: `.agents/references/research/final-implementation-import/grill-session/022-c4os-grill-question-022-mutating-tool-defaults.json`

  - What should trusted-project file writes default to?: Allow implied trusted-project writes; ask destructive deletes or broad rewrites
  - What should outside-project file writes default to?: Ask by default unless explicitly requested in the current task

### DEC-010: 028: C4OS Grill Question 028 - Branch Selection

Source: `.agents/references/research/final-implementation-import/grill-session/028-c4os-grill-question-028-branch-selection.json`

  - When should the chat prompt branch button appear?: Only when the active project folder is a Git repository
  - What should Choose/Create Branch do?: Create/select project branch for separate accepted work; existing chat thread branch remains read-only

### DEC-011: 029: C4OS Grill Question 029 - Remove Project Semantics

Source: `.agents/references/research/final-implementation-import/grill-session/029-c4os-grill-question-029-remove-project-semantics.json`

  - What should Remove Project do?: Remove from current workspace only; preserve project chat history
  - What should Remove Chat do?: Confirm, then permanently remove chat and C4OS-owned history

### DEC-012: 030: C4OS Grill Question 030 - Clone Repository Registration

Source: `.agents/references/research/final-implementation-import/grill-session/030-c4os-grill-question-030-clone-repository-registration.json`

  - How should Clone Repository execute?: C4OS Tauri tool wraps system Git, then registers destination folder
  - After clone succeeds, what should C4OS do?: Add cloned folder to current workspace and select it active

### DEC-013: 031: C4OS Grill Question 031 - Project Search

Source: `.agents/references/research/final-implementation-import/grill-session/031-c4os-grill-question-031-project-search.json`

  - What should Search Projects search?: Chat threads only
  - How should search results appear?: Center-screen takeover with X close button
