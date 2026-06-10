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
- [x] Status grid includes wrapping/overflow guards for default-window content.
- [x] Dev server root route serves `index.html`.
- [x] Temporary icon loads without Tauri setup failure.
- [x] No Windows support claim appears in release notes.

## Known MVP Caveats

- Temporary icon, not final brand artwork.
- Provider/project/session UI surfaces are MVP scaffolds and backend
  projections, not a polished end-user workflow.
- Windows compatibility should be validated before expanding the release target.
