# macOS Native WebKit Production-Lock Revalidation — 2026-07-22

## Scope

Task 00010 revalidated the Frozen public-WebKit feasibility boundary against the production dependency lock before adopting it in application code. This remains isolated Proof evidence, not production Browser verification.

## Locked target

- macOS 26.5.2, arm64
- Tauri 2.11.5
- tauri-build 2.6.3
- objc2-web-kit 0.3.2
- public WebKit APIs only

## Harness repair

The Task 00001 root Cargo workspace initially captured the older standalone Proof and prevented it from building. An empty Proof-local `[workspace]` restored isolation. The Proof lock and deterministic version assertions were then aligned from Tauri 2.11.2/tauri-build 2.6.2 to the exact production versions above.

## Commands and result

```sh
cargo build --offline --manifest-path proofs/macos-wkwebview-production-boundary/Cargo.toml
cargo run --offline --manifest-path proofs/macos-wkwebview-production-boundary/Cargo.toml
node --test proofs/macos-wkwebview-production-boundary/proof.test.mjs
cargo run --offline --manifest-path proofs/macos-wkwebview-production-boundary/Cargo.toml
node --test proofs/macos-wkwebview-production-boundary/proof.test.mjs
```

Both consecutive live native runs passed all eight boundary checks. Both deterministic result verifications passed 2/2 after the version assertion was aligned. The final raw result records `Prompt` media handling, no page IPC/custom profile path, native child attach/resize/focus, sanitized controller events, persistent profile isolation, ephemeral destruction, and scope-local clearing.

## Boundary

The production implementation must still prove its own controller lifecycle, geometry, profile registry, clearing recovery, artifact integration, permissions, hostile-page isolation, accessibility, and packaged native behavior. This rerun only confirms that the selected public-WebKit approach remains feasible on the exact production Tauri lock.
