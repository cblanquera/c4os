# C4OS

C4OS is a local-first desktop control center for one coding-focused AI agent
session in one selected Git project at a time.

## Current MVP Scope

- macOS Apple Silicon release target.
- Tauri desktop shell.
- Local SQLite metadata.
- OS credential storage for provider keys.
- OpenRouter provider setup boundary.
- OpenCode runtime supervision boundary.
- Project-root file access, approvals, tool timeline, Git diffs, and text-like
  artifacts.

Windows 11 x64 validation was re-scoped out of this MVP release. Validate on
Windows before expanding the release to Windows users.

## Requirements

- Node.js 20+ recommended.
- Rust/Cargo 1.96.0 or newer.
- macOS Apple Silicon for the current release target.

## Install

```bash
npm install
```

## Run In Development

```bash
npm run tauri -- dev
```

The Tauri dev command starts the static frontend server at
`http://127.0.0.1:5173` and launches the desktop app.

## Test

```bash
npm test
cargo test --manifest-path src-tauri/Cargo.toml
```

The macOS keychain smoke test is intentionally ignored by default because it
writes and deletes a dummy credential in the local keychain.

## Build

```bash
npm run tauri -- build
```

The release binary is produced at:

```text
src-tauri/target/release/c4os
```

## Icon

The current app icon is a temporary valid 512x512 PNG. Regenerate it with:

```bash
node scripts/generate-icon.mjs
```

Replace it with final brand artwork before a polished public release.
