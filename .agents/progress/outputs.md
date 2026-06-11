# Outputs

| ID | Output | Status | Verification |
| --- | --- | --- | --- |
| item-001 | App scaffold and backend command boundary | verified | `npm test`, `npm run build`, `npm audit --audit-level=moderate`, `npm run tauri -- --version`, and `npm run tauri -- build` pass. |
| item-002 | SQLite schema and migrations | verified | `cargo test --manifest-path src-tauri/Cargo.toml`, `npm test`, `npm run build`, and `npm run tauri -- build` pass. |
| item-003 | Keychain credential store | verified | `cargo test --manifest-path src-tauri/Cargo.toml`, ignored macOS keychain smoke test, `npm test`, and `npm run tauri -- build` pass. |
| item-004 | OpenRouter provider setup | verified | `cargo test --manifest-path src-tauri/Cargo.toml`, `npm test`, and `npm run tauri -- build` pass. |
| item-005 | Active-session credential reference rules | verified | `cargo test --manifest-path src-tauri/Cargo.toml`, `npm test`, and `npm run tauri -- build` pass. |
| item-006 | Project registration and Git status | verified | `cargo test --manifest-path src-tauri/Cargo.toml`, `npm test`, and `npm run tauri -- build` pass. |
| item-007 | Project-root path resolver | verified | `cargo test --manifest-path src-tauri/Cargo.toml`, `npm test`, and `npm run tauri -- build` pass. |
| item-008 | Read-only project browser and AGENTS display | verified | `cargo test --manifest-path src-tauri/Cargo.toml`, `npm test`, and `npm run tauri -- build` pass. |
| item-009 | Hardened OpenCode runtime launcher | verified | `cargo test --manifest-path src-tauri/Cargo.toml`, `npm test`, and `npm run tauri -- build` pass. |
| item-010 | Runtime event stream normalizer | verified | `cargo test --manifest-path src-tauri/Cargo.toml`, `npm test`, and `npm run tauri -- build` pass. |
| item-011 | Instruction preflight and disclosure | verified | `cargo test --manifest-path src-tauri/Cargo.toml`, `npm test`, and `npm run tauri -- build` pass. |
| item-012 | Runtime stop and crash interruption mapping | verified | `cargo test --manifest-path src-tauri/Cargo.toml`, `npm test`, and `npm run tauri -- build` pass. |
| item-013 | Protected action classifier | verified | `cargo test --manifest-path src-tauri/Cargo.toml`, `npm test`, and `npm run tauri -- build` pass. |
| item-014 | Approval prompt state and ledger | verified | `cargo test --manifest-path src-tauri/Cargo.toml`, `npm test`, and `npm run tauri -- build` pass. |
| item-015 | Session allow and structured denial results | verified | `cargo test --manifest-path src-tauri/Cargo.toml`, `npm test`, and `npm run tauri -- build` pass. |
| item-016 | File tool policy execution | verified | `cargo test --manifest-path src-tauri/Cargo.toml`, `npm test`, and `npm run tauri -- build` pass. |
| item-017 | Shell executor and environment filter | verified | `cargo test --manifest-path src-tauri/Cargo.toml`, `npm test`, and `npm run tauri -- build` pass. |
| item-018 | Shell output summary policy | verified | `cargo test --manifest-path src-tauri/Cargo.toml`, `npm test`, and `npm run tauri -- build` pass. |
| item-019 | Tool timeline and ledger UI | verified | `cargo test --manifest-path src-tauri/Cargo.toml`, `npm test`, and `npm run tauri -- build` pass. |
| item-020 | Session transcript and runtime state UI | verified | Rust session projection tests, `npm test`, and native Tauri build pass. |
| item-021 | Approval prompt UI | verified | Rust approval projection tests, `npm test`, and native Tauri build pass. |
| item-022 | Stop retry and restart recovery UX | verified | Rust recovery tests, `npm test`, and native Tauri build pass. |
| item-023 | Changed files and diff viewer | verified | Git diff tests, `npm test`, and native Tauri build pass. |
| item-024 | Text-like artifact records and viewer | verified | Artifact persistence tests, UI tests, and native Tauri build pass. |
| item-025 | MVP technical test suite | verified | Full automated local technical suite passes. |
| item-026 | App-build and product QA gates | verified | macOS Apple Silicon local build and automated QA pass; Windows 11 x64 validation re-scoped out of public MVP release. |
| item-027 | Stabilization and release prep | verified | `npm test`, `npm run build`, `cargo test --manifest-path src-tauri/Cargo.toml`, and native Tauri build pass. |
| item-028 | Project session catalog foundation | verified | Rust session catalog tests, JS tests, frontend build, and native Tauri build pass. |
| item-029 | Project-scoped session selection | verified | Rust selected-session tests, JS tests, frontend build, and native Tauri build pass. |
| item-030 | Session rename pin and archive foundation | verified | Migration tests, metadata API tests, JS tests, frontend build, and native Tauri build pass. |
| item-031 | Project selector state foundation | verified | Registered-project tests, selector persistence tests, JS tests, frontend build, and native Tauri build pass. |
| item-032 | V1 worktree lifecycle scope | verified | Planning decision verified by roadmap, non-goals, readiness gaps, and Git acceptance updates. |
| item-033 | Project selector status surface | verified | `npm test`, `npm run build`, `npm run tauri -- build`, and `cargo test --manifest-path src-tauri/Cargo.toml` pass. |
| item-034 | V1 nested AGENTS resolution scope | verified | `cargo test --manifest-path src-tauri/Cargo.toml instruction_preflight`, `npm test`, `npm run build`, `cargo test --manifest-path src-tauri/Cargo.toml`, and native Tauri build pass. |
