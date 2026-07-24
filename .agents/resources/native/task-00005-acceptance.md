# Task 00005 Native Acceptance Evidence

Date: 2026-07-21

Target: macOS arm64, production-composed debug `C4OS.app`

Checkpoint base: `3e06b0b`

## Golden path

The smallest production path is Rust-owned and target-qualified:

1. The decorated native window captures a source-qualified macOS appearance snapshot before reveal.
2. The renderer consumes that snapshot, applies semantic Light/Dark tokens, and independently observes later `prefers-color-scheme` changes without persisting an override.
3. The native application menu emits the typed Settings command and `Cmd+,` routes through the live application router.
4. Rust-owned file/folder/workspace pickers return only one-use opaque grants; the renderer never receives a selected path.
5. Runtime asset and Keychain initialization run behind a fail-closed atomic publication holder so platform UI startup does not expose a partial production runtime.

## Automated verification

All commands ran from the repository root with writers frozen. Cargo commands were serialized.

| Gate | Result |
| --- | --- |
| `npm run format:check` | passed |
| `npm run lint` | passed with zero warnings |
| `npm run typecheck` | passed |
| `npm run test:unit` | 15 files, 62/62 passed |
| `npm run test:e2e` | 16/16 passed |
| `cargo fmt --all -- --check` | passed in 0.85s |
| `cargo test --lib atomic_capability_publication_tests -- --nocapture --test-threads=1` | 2/2 passed; 38.9s including compilation |
| `cargo clippy --workspace --all-targets -- -D warnings` | passed; 49.4s |
| `cargo test --workspace --all-targets -- --test-threads=1` | all runnable tests passed with no failures; only explicitly opt-in loopback, live-Keychain, native, and bundle tiers ignored |
| `cargo audit` | no vulnerabilities; 17 upstream maintenance warnings retained as visible dependency inputs |
| `npm run tauri:build` | passed; exact debug app bundled and Task-owned sidecars copied in about 114s |

The platform integration suite contributes 11 passing Rust tests. Renderer coverage includes the semantic foundation, native transport, live theme, route, dialog, focus, responsive, reduced-motion, overflow, picker cancellation/grant, and remount state cases.

## Native macOS matrix

The exact rebuilt app at `target/debug/bundle/macos/C4OS.app` was exercised through native Computer Use.

| Check | Result |
| --- | --- |
| Fresh and repeated launch | passed; standard decorated window and native application menu present on both launches |
| First visible theme | passed in Dark with `macOS appearance` source and no stored override |
| Live appearance switch | passed Dark to Light in place; source changed to `Webview live preference`; reviewed state was retained; system was restored to Dark |
| Native menu | passed: C4OS, File, Edit, View, Window, Help |
| Settings shortcut | passed: `Cmd+,` opened `tauri://localhost#/settings/providers` |
| Back restoration | passed through the live router; the exact Dirty review state survived Settings and route remount |
| Folder picker selection | passed; renderer showed only `Access granted to c4os.` and no selected path |
| Folder picker cancellation | passed without grant or state corruption |
| Minimum width | passed at approximately 623px with internal vertical scrolling and no document-level horizontal overflow |
| Narrow Settings | passed with compressed navigation and independently scrolling content |
| Dialog containment | passed with inert background, Escape dismissal, and focus restoration to `Review dialog` |
| Accessibility inspection | passed for named buttons/headings, native window controls/menu, focus ownership, and route identity |
| Reduced motion and contrast/token checks | passed in automated renderer matrices |

After the final Rust-only startup composition change, the rebuilt app was launched twice from a confirmed `isRunning: false` state. The first successful accessibility snapshots were available within 12.4s and 15.8s respectively; Computer Use itself transiently timed out during the first probe. Once visible, `Cmd+,` opened Settings and Back restored the exact prior route in under 1.3s. A targeted stack sample showed the macOS main thread in the AppKit event loop while production runtime verification remained on `c4os-production-runtime-initialization`; runtime commands remain fail-closed until one complete application value is published.

## Review artifacts

- `task-00005-final-repeated-launch.png` — repeated Dark launch and semantic state matrix.
- `task-00005-final-roundtrip.png` — Dirty state retained after native Settings roundtrip.
- `task-00005-light-live-min-width.png` — live Light switch at minimum width.
- `task-00005-dark-min-width.png` — Dark minimum-width composition.
- `task-00005-dark-settings-min-width.png` — compressed Settings navigation.
- `task-00005-native-picker-grant-exact.png` — opaque selection success with no raw path.
- `task-00005-dark-dialog-exact.png` — modal containment and semantic feedback states.

## Limitations and deferred integration

- These are Task 00005 foundation surfaces. Tasks 00006, 00007, 00008, 00013, and 00015B still own the complete shell, Chat/artifact composition, full Settings content, and final cross-surface native/accessibility audit.
- The full native runtime golden path remains Task 00004 evidence and is not substituted by this platform UI matrix.
- The debug bundle is intentionally large and has measurable first-accessibility latency. No Frozen numeric startup budget exists for Task 00005; the synchronous authority defect was removed, partial runtime publication is prohibited, and the remaining timing is recorded rather than concealed.
- No signing, notarization, distribution, push, or pull request occurred.
