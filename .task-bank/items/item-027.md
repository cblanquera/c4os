# item-027: Sprint 4 Stabilization And Release Prep

## Status

verified

## Objective

Turn the verified MVP implementation into a clean runnable baseline by
documenting the run/build flow, protecting the dev-server route regression, and
recording release handoff details.

## Deliverables

- README with install, dev, test, and build commands.
- Dev-server regression coverage for the root app route.
- macOS Apple Silicon release checklist.
- Final verification gate run.

## Verification

- `npm test` passed.
- `npm run build` passed.
- `cargo test --manifest-path src-tauri/Cargo.toml` passed.
- `npm run tauri -- build` passed.
