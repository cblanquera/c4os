# File Editor Plugin Decisions

Status: proposed

### DEC-001: 032: C4OS Grill Question 032 - File Explorer Add To Chat

Source: `.agents/references/research/final-implementation-import/grill-session/032-c4os-grill-question-032-file-explorer-add-to-chat.json`

  - What should File Explorer `Add to chat` attach?: should inline paste the same as prompt tagging ie. `@bin.js`
  - What should Copy Path copy?: Absolute filesystem path

### DEC-002: 033: C4OS Grill Question 033 - File Icons Scope

Source: `.agents/references/research/final-implementation-import/grill-session/033-c4os-grill-question-033-file-icons-scope.json`

  - Which file icons should the IDE plugin support?: Full VS Code-like file icon theme
  - Should hidden files remain visible in the IDE File Explorer?: Hidden files visible except `.git`

### DEC-003: 046: C4OS Grill Question 046 - IDE File Operation Boundary

Source: `.agents/references/research/final-implementation-import/grill-session/046-c4os-grill-question-046-ide-file-operation-boundary.json`

  - Should the IDE plugin own save and revert UI?: IDE owns save/revert UI and contributes OS menu commands when active
  - Should file create, rename, and delete be part of the IDE plugin scope?: Include create, rename, and guarded delete with confirmation
  - If IDE delete is included, what should delete do?: Confirm and move to OS trash/recycle when available
  - How should IDE handle external file changes?: Detect changes, show conflict state, and require user choice before overwrite

### DEC-004: Backend App Architect Resolution - File Editor

Source: 2026-07-02 user architect profile in active chat.

  - Authority boundary: The File Editor plugin owns the editing UI but file
    reads, writes, creates, renames, and deletes execute through C4OS file
    services and tool/file policy. The plugin never bypasses backend policy
    through direct renderer filesystem access.
  - Event model: Open, dirty, save, revert, external-change, conflict,
    create, rename, delete-to-trash, and prompt-reference insertion emit typed
    C4OS events so shell, Chat Debug, and prompt interactions can hydrate
    without parsing editor UI text.
  - Windows compatibility: Delete uses OS trash/recycle through a platform
    adapter when available, with visible fallback when recycle is unavailable.
    Rename, hidden-file display, file icons, path casing, and external-change
    detection must account for Windows filesystem behavior.
  - Worker-friendly UX: The plugin may be called File Editor or Files depending
    on shell copy; behavior must support text/config/docs/ops files, not only
    source code.
  - Maintainability: Explorer tree, editor buffer state, file operations,
    conflict detection, icon mapping, OS menu commands, and prompt insertion
    are separate modules with documented state transitions.

### DEC-005: POC Result - IDE File Operation Boundary

Source: `proofs/ide-file-operation-boundary/`

  - The File Editor plugin owns editing UX, but open, save, create, rename,
    and delete-to-trash execute through backend file services and policy gates.
  - External file changes produce conflict state and require explicit user
    choice before overwrite.
  - Add to chat inserts a prompt `@file` reference rather than hidden
    attachments or full file contents.
  - Editor lifecycle and file operations emit typed events for shell, prompt,
    and Chat Debug hydration.
