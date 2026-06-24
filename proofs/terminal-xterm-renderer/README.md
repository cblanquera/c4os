# Terminal Xterm Renderer Proof

Status: proof artifact
Updated: 2026-06-24

## Purpose

Validate `@xterm/xterm` as the browser-side renderer for the C4OS Terminal tab
before changing production UI.

## Setup

```sh
cd proofs/terminal-xterm-renderer
npm install
```

## Review

Open `index.html` directly in a browser.

- Type normally to enter a command.
- Press Enter to submit.
- Press Tab to cycle sample commands.

## Scope

This proof simulates backend output. It does not execute commands and does not
wire Tauri commands/events.
