# Agent Progress Manifest

| ID | Type | Status | Owner | Output | Notes |
| --- | --- | --- | --- | --- | --- |
| item-001 | implementation | verified | codex | App scaffold and backend boundary | JS tests/build/audit and native Tauri build pass. |
| item-002 | implementation | verified | codex | SQLite schema and migrations | Rust migration tests and native build pass. |
| item-003 | implementation | verified | codex | Keychain credential store | Fake-store tests, macOS keychain smoke, and native build pass. |
| item-004 | implementation | verified | codex | OpenRouter provider setup | Provider setup tests and native build pass. |
| item-005 | implementation | verified | codex | Active-session credential reference rules | Stable session credential/model tests and native build pass. |
| item-006 | implementation | verified | codex | Project registration and Git status | Temp Git repo tests and native build pass. |
| item-007 | implementation | verified | codex | Project-root path resolver | Traversal, symlink, and secret-deny tests pass. |
| item-008 | implementation | verified | codex | Read-only project browser and AGENTS display | Browser and AGENTS tests pass. |
| item-009 | implementation | verified | codex | Hardened OpenCode runtime launcher | Supervised spawn and launch failure tests pass. |
| item-010 | implementation | verified | codex | Runtime event stream normalizer | Fixture event mapping tests pass. |
| item-011 | implementation | verified | codex | Instruction preflight and disclosure | Inventory, redaction, and disclosure persistence tests pass. |
| item-012 | implementation | verified | codex | Runtime stop and crash interruption mapping | Stop and restart interruption tests pass. |
| item-013 | implementation | verified | codex | Protected action classifier | Action category and boundary tests pass. |
| item-014 | implementation | verified | codex | Approval prompt state and ledger | Durable answered and non-durable pending tests pass. |
| item-015 | implementation | verified | codex | Session allow and structured denial results | Session allow and denial tests pass. |
| item-016 | implementation | verified | codex | File tool policy execution | Read/write policy and batch tests pass. |
| item-017 | implementation | verified | codex | Shell executor and environment filter | Environment filtering and shell execution tests pass. |
| item-018 | implementation | verified | codex | Shell output summary policy | Redaction/truncation/omission tests pass. |
| item-019 | implementation | verified | codex | Tool timeline and ledger UI | Timeline projection and UI smoke tests pass. |
| item-020 | implementation | verified | codex | Session transcript and runtime state UI | Session projection tests, JS tests, and native build pass. |
| item-021 | implementation | verified | codex | Approval prompt UI | Prompt projection tests, JS tests, and native build pass. |
| item-022 | implementation | verified | codex | Stop retry and restart recovery UX | Recovery tests, JS tests, and native build pass. |
| item-023 | implementation | verified | codex | Changed files and diff viewer | Git diff tests, JS tests, and native build pass. |
| item-024 | implementation | verified | codex | Text-like artifact records and viewer | Artifact persistence tests, JS tests, and native build pass. |
| item-025 | verification | verified | codex | MVP technical test suite | Full local automated suite passes. |
| item-026 | verification | verified | codex | App-build and product QA gates | macOS Apple Silicon local gates pass; Windows 11 x64 validation explicitly re-scoped out of public MVP release. |
| item-027 | stabilization | verified | codex | Stabilization and release prep | README, dev-server regression coverage, release checklist, and full gates pass. |
| item-028 | implementation | verified | codex | Project session catalog foundation | Session catalog tests and full local gates pass. |
| item-029 | implementation | verified | codex | Project-scoped session selection | Selection ownership tests and full local gates pass. |
| item-030 | implementation | verified | codex | Session rename pin and archive foundation | Migration, metadata APIs, Rust tests, JS tests, web build, and native Tauri build pass. |
| item-031 | implementation | verified | codex | Project selector state foundation | Selector tests, Rust tests, JS tests, web build, and native Tauri build pass. |
| item-032 | planning | verified | codex | V1 worktree lifecycle scope | Sprint 9 deferred worktree creation and cleanup beyond current V1; later V1 items still need their own accepted scope and acceptance criteria. |
| item-033 | implementation | verified | codex | Project selector status surface | JS tests, Rust tests, web build, and native Tauri build pass. |
| item-034 | implementation | verified | codex | V1 nested AGENTS resolution scope | Ordered display-guidance tier implemented; Rust tests, JS tests, web build, and native Tauri build pass. |
| item-035 | planning | verified | codex | V1 Agent Skills discovery and invocation scope | Explicit discovery/invocation-only tier accepted on 2026-06-12. |
| item-036 | implementation | verified | codex | Explicit project-local skill discovery and invocation | Rust tests, JS tests, web build, full Rust tests, and native Tauri build pass. |
| item-037 | planning | verified | codex | V1 local stdio MCP scope | Local-stdio explicit-approval-only tier accepted on 2026-06-12. |
| item-038 | implementation | verified | codex | Local stdio MCP explicit approval surface | Rust MCP tests, JS tests, web build, full Rust tests, and native Tauri build pass. |
| item-039 | planning | verified | codex | V1 rich artifact preview scope | `raster_image_preview_only` accepted on 2026-06-12. |
| item-040 | implementation | verified | codex | Passive raster image artifact previews | Artifact tests, JS tests, web build, full Rust tests, and native Tauri build pass. |
| item-041 | implementation | verified | codex | Project JSON export with import deferred | Export tests, JS tests, web build, full Rust tests, and native Tauri build pass. |
| item-042 | planning | verified | codex | V1 retention and session delete scope | `archived_session_delete_only` accepted on 2026-06-12. |
| item-043 | implementation | verified | codex | Archived session delete | Rust tests, JS tests, web build, full Rust tests, and native Tauri build pass. |
| item-044 | planning | verified | codex | V1 long-term memory scope | `no_durable_memory_v1` accepted on 2026-06-12. |
| item-045 | planning | verified | codex | V1 broader compatibility claims scope | Compatibility status tests, JS tests, web build, full Rust tests, and native Tauri build pass. |
| item-046 | planning | verified | codex | V1 provider expansion scope | Provider expansion status tests, JS tests, web build, full Rust tests, and native Tauri build pass. |
| item-047 | planning | verified | codex | V1 browser and web-viewing scope | Browser/web status tests, JS tests, web build, full Rust tests, and native Tauri build pass. |
| item-048 | planning | verified | codex | V1 plugin and marketplace scope | Plugin/marketplace status tests, JS tests, web build, full Rust tests, and native Tauri build pass. |
