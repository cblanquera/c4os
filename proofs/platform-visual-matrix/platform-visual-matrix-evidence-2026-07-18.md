# Platform Visual Acceptance Matrix Evidence — 2026-07-18

## Environment

- macOS 26.5.1 (25F80), arm64
- Tauri 2.11.5, tao 0.35.3, wry 0.55.1
- WKWebView user agent family: `AppleWebKit/605.1.15`
- Windows and Linux: `not run`

## Commands

```sh
node --test proofs/platform-visual-matrix/proof.test.mjs
cargo check --manifest-path proofs/platform-visual-matrix/Cargo.toml
```

The release `.app` bundle was built and reviewed in macOS Dark and Light.

## Observed Signal

- 7 static fixture, semantic-token, forced-colors, reduced-motion, focus, and containment tests passed.
- Shell, transcript, artifact, editor, terminal, Settings, popover, and dialog fixtures rendered in both appearances.
- The in-app audit reported 23/23 runtime checks passing in Dark and again in Light, including fixture presence, explicit themed surfaces, document containment, and editor/terminal local overflow containment.
- At the configured minimum window size, the matrix reflowed to one content column, retained document-level containment, and continued to report 23/23 checks passing.
- The visible scheme label followed the live macOS appearance change without reload.
- The popover opened from its trigger, dismissed with Escape, and returned focus to the trigger.
- The dialog placed focus in its first field and dismissed with Escape.
- Focus indicators, non-color status symbols, semantic system colors, forced-colors overrides, and reduced-motion overrides are explicit in the proof source.

## Result

**Passed on the tested macOS matrix.** This establishes the deterministic representative surface contract on this target, not full production UI acceptance. Windows forced colors, Windows native rendering, GNOME/Wayland, KDE, X11, Linux accessibility preferences, and other unavailable targets remain `not run`.
