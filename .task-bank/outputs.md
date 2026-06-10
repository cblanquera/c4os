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
