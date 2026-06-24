# Terminal Inline UI POC Evidence

Date: 2026-06-24
Status: passed-static-smoke-ready-for-review
Scope: disposable UI proof only

## Finding

The current TASK-011 production UI is not the intended terminal interaction
model. It uses a separate command form and split output panes. User feedback
requests a terminal transcript model where command input, submitted command
text, command output, and the next prompt/cursor all live in one continuous
terminal stream.

## Tauri CLI Plugin Check

C4OS is not using Tauri's CLI plugin for TASK-011. Repository search found no
`tauri-plugin-cli`, `@tauri-apps/plugin-cli`, or `tauri_plugin_cli` usage in
the current backend or frontend sources.

The Tauri v2 CLI plugin is not the right proof target for the reviewed UI gap.
It parses arguments passed to the application process. It does not provide an
embedded interactive terminal emulator, PTY lifecycle, renderer transcript, or
inline cursor behavior.

## Proof Artifact

- `proofs/terminal-inline-ui/index.html`

## Review Notes

- Open `index.html` directly in a browser.
- Type normally to enter a command.
- Press Enter to append command output.
- Press Tab to cycle sample commands: `ls`, `rm -rf .git`, `git init`,
  `yarn install`.

## Smoke Check

Command:

```sh
node -e "import('playwright').then(async ({ chromium }) => { const browser = await chromium.launch(); const page = await browser.newPage({ viewport: { width: 1220, height: 760 } }); await page.goto('file://' + process.cwd() + '/proofs/terminal-inline-ui/index.html'); await page.keyboard.press('Tab'); await page.keyboard.press('Enter'); await page.keyboard.press('Tab'); await page.keyboard.press('Enter'); const text = await page.locator('#screen').innerText(); if (!text.includes('cblanquera@Chris-MacBook-2022 c4os % ls')) throw new Error('missing inline ls'); if (!text.includes('blocked: high-impact command requires approval before launch')) throw new Error('missing inline blocked output'); if ((await page.locator('.cursor').count()) !== 1) throw new Error('missing cursor'); await page.screenshot({ path: '/private/tmp/c4os-terminal-inline-ui-proof.png', fullPage: true }); await browser.close(); })"
```

Result: passed.

Screenshot: `/private/tmp/c4os-terminal-inline-ui-proof.png`

## Expected Product Promotion

If accepted, TASK-011 production UI should replace the command-form/split-pane
Terminal tab with a transcript-driven terminal surface:

- one terminal viewport;
- prompt and cursor in the transcript;
- submitted command remains inline;
- output appends directly after command;
- next prompt and cursor stay at the bottom;
- agent command terminal can remain product-owned as a separate transcript mode
  or read-only transcript, but should not force a split-pane user terminal.

## Limits

- This proof simulates command output in static JavaScript.
- It does not execute commands.
- It does not replace the existing backend-owned terminal records.
- Real PTY lifecycle remains covered by:
  - `proofs/native-terminal-plugin/`
  - `proofs/native-terminal-tauri/`
