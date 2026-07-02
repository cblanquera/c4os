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
