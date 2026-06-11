# MVP Release Checklist

Date: 2026-06-11

## Release Scope

- Target: macOS Apple Silicon.
- Windows 11 x64: explicitly re-scoped out of this MVP release.
- Linux: deferred.

## Required Local Gates

- [x] `npm test`
- [x] `cargo test --manifest-path src-tauri/Cargo.toml`
- [x] `npm run build`
- [x] `npm run tauri -- build`
- [x] Launch `npm run tauri -- dev` and confirm the app shell appears, not
  `Not found`.
- [x] Confirm `src-tauri/target/release/c4os` exists after release build.

## Manual Smoke

- [x] App shell opens.
- [x] First-run UI exposes OpenRouter key entry, model selection, Git project
  path entry, and prompt submission controls.
- [x] Status grid includes wrapping/overflow guards for default-window content.
- [x] Dev server root route serves `index.html`.
- [x] Temporary icon loads without Tauri setup failure.
- [x] No Windows support claim appears in release notes.

## Known MVP Caveats

- Temporary icon, not final brand artwork.
- Provider/project/session UI surfaces expose a testable first-run workflow.
  Prompt submission now runs through the backend OpenCode JSON runner and
  persists assistant output when the runtime/provider call succeeds. Prompt
  submission now shows an immediate pending state and the OpenCode process is
  bounded by a timeout. Command rejections are normalized into visible UI
  notice text, and duplicate prompt starts are blocked while a backend session
  is active.
- Startup recovery marks persisted active sessions as interrupted so a
  force-closed app does not reopen with a stale `Session running` blocker.
- This MVP path allows passive project inspection through OpenCode and denies
  mutation/shell/network tools until the live approval bridge is connected.
- Real OpenRouter end-to-end validation still requires a valid user key,
  network access, and a selected model route that OpenRouter accepts.
- Windows compatibility should be validated before expanding the release target.
