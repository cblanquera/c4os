# TASK-011 Xterm PTY Bridge POC

Status: passed-on-macos-ready-for-review
Updated: 2026-06-24
Phase: poc

## Proof Question

Can C4OS connect `@xterm/xterm` in the browser to a Rust `portable-pty`
backend through a small local bridge, proving input, output, resize, and
terminal transcript behavior together before production implementation?

## Why It Matters

The renderer-only and PTY-only proofs passed independently. Production TASK-011
still needs the combined behavior: user keystrokes from xterm must reach the
PTY, PTY output must stream back into xterm, resize must reach the PTY, and the
visible UI must remain a native-terminal-style transcript.

## Expected Proof

- A proof-local Rust binary starts a local HTTP server and a `portable-pty`
  shell in the trusted project root.
- The browser page mounts `@xterm/xterm`.
- PTY output streams into xterm.
- xterm input posts back to the PTY.
- xterm/container resize posts to the backend and reaches the PTY.
- Browser automation can verify a submitted command appears with PTY output in
  the xterm transcript.

## Failure Signal

- xterm renders but does not receive real PTY output.
- Browser input does not reach the PTY.
- Resize cannot be forwarded to the PTY.
- The bridge requires a production route, settings abstraction,
  React/Preact/JSX, or bundler.
- The proof cannot keep the PTY scoped to the trusted project root.

## Verification Method

- Install proof-local xterm dependency in `proofs/terminal-xterm-pty-bridge/`.
- Run
  `cargo run --quiet --manifest-path proofs/terminal-xterm-pty-bridge/Cargo.toml`.
- Open the printed local URL.
- Use browser automation to send `printf bridge-ok` through xterm and verify
  the xterm transcript contains `bridge-ok`.

## Result

The integrated bridge proof passed on macOS on 2026-06-24. A proof-local Rust
server launched a `portable-pty` shell in the trusted project root, served an
`@xterm/xterm` browser UI, streamed PTY output to xterm through server-sent
events, accepted xterm-originated input through the bridge, accepted a resize
request, and rendered real PTY output in the xterm transcript.

## Decision Target

If accepted, promote the architecture pattern: xterm renderer in the existing
right-panel Terminal tab, Rust `portable-pty` session manager behind C4OS-owned
Tauri commands/events, and product-owned terminal/session/action records.
