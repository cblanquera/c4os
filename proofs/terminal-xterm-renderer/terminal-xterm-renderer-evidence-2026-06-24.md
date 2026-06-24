# Terminal Xterm Renderer POC Evidence

Date: 2026-06-24
Status: passed-static-smoke-ready-for-review
Scope: proof-only renderer validation

## Proof Artifact

- `proofs/terminal-xterm-renderer/index.html`

## Intended Verification

```sh
cd proofs/terminal-xterm-renderer
npm install
```

Then run a browser smoke check that opens `index.html`, cycles sample commands
with Tab, submits them with Enter, and verifies that the xterm viewport contains
the inline prompt, submitted command, output, and next prompt/cursor.

## Verification Run

Dependency install:

```sh
cd proofs/terminal-xterm-renderer
npm install
```

Result: passed, installed `@xterm/xterm`, 0 vulnerabilities.

Browser smoke:

```sh
node -e "import('playwright').then(async ({ chromium }) => { const browser = await chromium.launch(); const page = await browser.newPage({ viewport: { width: 1220, height: 760 } }); await page.goto('file://' + process.cwd() + '/proofs/terminal-xterm-renderer/index.html'); await page.waitForFunction(() => window.__c4osXtermProof); await page.evaluate(() => { window.__c4osXtermProof.submitCommand('ls'); window.__c4osXtermProof.submitCommand('rm -rf .git'); }); const text = await page.locator('.xterm-screen').innerText(); if (!text.includes('cblanquera@Chris-MacBook-2022 c4os % ls')) throw new Error('missing inline ls command: ' + text); if (!text.includes('blocked: high-impact command requires approval before launch')) throw new Error('missing blocked output: ' + text); if (!text.includes('cblanquera@Chris-MacBook-2022 c4os % rm -rf .git')) throw new Error('missing inline blocked command: ' + text); await page.screenshot({ path: '/private/tmp/c4os-terminal-xterm-renderer-proof.png', fullPage: true }); await browser.close(); })"
```

Result: passed.

Screenshot: `/private/tmp/c4os-terminal-xterm-renderer-proof.png`

## Decision Target

If the smoke check passes, `@xterm/xterm` is a viable candidate for the
production Terminal tab renderer. Production execution should still go through
C4OS-owned Tauri commands/events.
