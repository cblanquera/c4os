# Terminal Xterm PTY Bridge POC Evidence

Date: 2026-06-24
Status: passed-on-macos-ready-for-review
Scope: integrated proof only

## Proof Artifact

- `proofs/terminal-xterm-pty-bridge/`

## Intended Verification

```sh
cd proofs/terminal-xterm-pty-bridge
npm install
```

```sh
cargo run --quiet --manifest-path proofs/terminal-xterm-pty-bridge/Cargo.toml
```

Then open the printed local URL and run a browser smoke test that verifies:

- PTY output reaches xterm.
- xterm input reaches PTY.
- resize can be posted to the backend.
- xterm transcript contains command and output from the real PTY path.

## Verification Run

Dependency install:

```sh
cd proofs/terminal-xterm-pty-bridge
npm install
```

Result: passed, installed `@xterm/xterm`, 0 vulnerabilities.

Rust formatting:

```sh
cargo fmt --manifest-path proofs/terminal-xterm-pty-bridge/Cargo.toml -- --check
```

Result: passed.

Bridge server:

```sh
cargo run --quiet --manifest-path proofs/terminal-xterm-pty-bridge/Cargo.toml
```

Result: passed server startup.

Observed server output:

```text
terminal-xterm-pty-bridge listening at http://127.0.0.1:52381
trusted root: /Users/cblanquera/server/projects/cblanquera/c4os
```

Browser smoke:

```sh
node -e "import('playwright').then(async ({ chromium }) => { const browser = await chromium.launch(); const page = await browser.newPage({ viewport: { width: 1220, height: 760 } }); await page.goto('http://127.0.0.1:52381/'); await page.waitForFunction(() => window.__c4osBridgeProof && document.querySelector('.xterm-screen')); await page.evaluate(async () => { await fetch('/resize', { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify({ rows: 40, cols: 120 }) }); await window.__c4osBridgeProof.send('printf bridge-ok\\n'); }); await page.waitForFunction(() => document.querySelector('.xterm-screen')?.innerText.includes('bridge-ok'), null, { timeout: 5000 }); const text = await page.locator('.xterm-screen').innerText(); if (!text.includes('bridge-ok')) throw new Error('missing PTY output: ' + text); await page.screenshot({ path: '/private/tmp/c4os-terminal-xterm-pty-bridge-proof.png', fullPage: true }); await browser.close(); })"
```

Result: passed.

Screenshot: `/private/tmp/c4os-terminal-xterm-pty-bridge-proof.png`

## Proven Together

- `portable-pty` shell started in trusted root.
- PTY output streamed into `@xterm/xterm`.
- Browser/xterm-originated input reached the PTY.
- Resize request reached the backend bridge.
- xterm transcript rendered real PTY output from the submitted command.

## Remaining Production Work

- Replace the proof HTTP/SSE bridge with C4OS-owned Tauri commands/events.
- Add session-owned lifecycle, persistence, action records, and cleanup.
- Apply approval policy gates before high-impact commands.
- Preserve the accepted Browser / Files / Terminal right-panel contract.

## Decision Target

If this proof passes, the combined `@xterm/xterm` plus `portable-pty` stack is
ready to be promoted into production design for TASK-011.
