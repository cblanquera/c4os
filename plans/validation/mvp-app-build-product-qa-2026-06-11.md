# MVP App-Build And Product QA Report

Date: 2026-06-11

## Environment

- Host: macOS arm64 (`Darwin ... RELEASE_ARM64_T6000 arm64`)
- Rust: `rustc 1.96.0 (ac68faa20 2026-05-25)`
- Cargo: `cargo 1.96.0 (30a34c682 2026-05-25)`
- Build target validated locally: macOS Apple Silicon

## Automated Gate Results

| Gate | Result | Evidence |
| --- | --- | --- |
| Rust unit/integration tests | Pass | `cargo test --manifest-path src-tauri/Cargo.toml` passed: 94 passed, 1 ignored keychain smoke. |
| JS smoke tests | Pass | `npm test` passed: 2 passed. |
| Web build | Pass | `npm run build` passed. |
| Native Tauri build | Pass | `npm run tauri -- build` passed and produced `src-tauri/target/release/c4os`. |

## MVP Journey Coverage

| Journey | Local QA Status | Notes |
| --- | --- | --- |
| Configure provider | Pass by automated coverage | Provider readiness, invalid-key fail-closed behavior, credential references, and active-session update guard are covered by Rust tests. |
| Register a local Git project | Pass by automated coverage | Temporary Git repository tests cover registration, dirty state, branch/status, and root `AGENTS.md` display. |
| Start one agent session | Pass by automated coverage | Runtime launcher, event normalization, one-active-session guard, and session transcript projection are covered. |
| Approve or deny an action | Pass by automated coverage | Approval ledger, prompt choices, destructive-command no-session-allow, denials, and policy blocks are covered. |
| Inspect agent changes | Pass by automated coverage | Modified, added, deleted, and untracked file diff states are covered with temporary Git repositories. |
| Resume context | Pass by automated coverage | SQLite persistence tests cover projects, sessions, messages, artifacts, and restart recovery behavior. |
| Recover from failure | Pass by automated coverage | Stop, interrupted restart state, retry-as-append, and failed runtime states are covered. |

## Text-Like Artifact Preview Report

| Artifact Type | Result | Notes |
| --- | --- | --- |
| Plain text | Pass | Stored under app-managed artifact layout and reopened after restart. |
| Markdown | Pass | Stored and listed as passive text-like artifact. |
| Logs | Pass | Stored with secret-shaped lines redacted. |
| Diffs | Pass | Stored and listed as passive text-like artifact. |
| Generated source/config | Pass | Source/config artifact types are accepted and listed. |
| Active HTML/executable content | Pass | Opened only as passive text; no active rendering or rich preview is exposed. |

## Release Platform Matrix

| Platform | Status | Evidence |
| --- | --- | --- |
| macOS Apple Silicon | Pass locally | Native Tauri build and automated QA gates pass on macOS arm64. |
| Windows 11 x64 | Re-scoped out of this MVP release | User approved public MVP release scope without Windows 11 x64 validation on 2026-06-11. |
| Linux x64 | Deferred | Deferred by MVP platform matrix. |

## Release Decision

Implementation and local macOS Apple Silicon validation passed.

Public MVP release may proceed under the approved re-scoped platform target:
macOS Apple Silicon only. Windows 11 x64 compatibility validation remains
recommended before expanding the release scope to Windows users.
