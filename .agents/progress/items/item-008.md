# item-008: Build Read-Only Project Browser And AGENTS Display

Status: verified
Type: implementation
Priority: high
Owner: codex

## Goal

Display project files read-only and show root `AGENTS.md` as passive
app-owned guidance.

## Inputs

- `.agents/archive/planning-corpus/acceptance/file-access.md`
- `.agents/archive/planning-corpus/acceptance/skills.md`
- `.agents/archive/planning-corpus/acceptance/project-management.md`

## Outputs

- Read-only file browser backend API.
- Root `AGENTS.md` passive display API.
- Secret-deny preview block.
- No project-wide content search UI.

## Constraints

- Browser opens must use the path resolver.
- Secret-deny files may be listed/detected but contents must not be previewed.
- Root `AGENTS.md` display must not change model context or permissions.

## Acceptance Criteria

- Allowed project files can be opened read-only.
- Outside-root file opens are blocked.
- Secret-deny preview content is blocked.
- Root `AGENTS.md` display is passive.

## Verification

- `cargo test --manifest-path src-tauri/Cargo.toml`
- `npm test`
- `npm run tauri -- build`

## Dependencies

- item-007

## Notes

- Verified on 2026-06-11 with read-only file open, outside-root block,
  secret-preview block, root `AGENTS.md` display, Node tests, and native Tauri
  build.
